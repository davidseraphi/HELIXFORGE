//! NATS JetStream integration with an in-process fallback for local/dev tests.
//!
//! Subject convention: `helix.<product>.<domain>.<event>`
//! Core subjects: `helix.core.<service>.<event>`

use bytes::Bytes;
use futures::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use shared_core::config::CoreConfig;
use shared_core::{HelixError, HelixResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;
use tracing::{info, warn};

pub const CORE_PREFIX: &str = "helix.core";

#[derive(Clone)]
pub struct HelixBus {
    backend: BusBackend,
}

#[derive(Clone)]
enum BusBackend {
    Memory(Arc<MemoryBus>),
    Nats(Arc<NatsBus>),
}

impl HelixBus {
    pub fn memory() -> Self {
        Self {
            backend: BusBackend::Memory(Arc::new(MemoryBus::new())),
        }
    }

    /// Connect using the URL-only legacy policy. Prefer [`connect_with_config`].
    pub async fn connect(nats_url: &str) -> HelixResult<Self> {
        Self::connect_with_policy(nats_url, true).await
    }

    /// Connect to NATS. When `allow_memory_fallback` is false, fails closed (Kimi P1).
    pub async fn connect_with_policy(
        nats_url: &str,
        allow_memory_fallback: bool,
    ) -> HelixResult<Self> {
        let mut cfg = CoreConfig::from_env("nats-client", 0)?;
        cfg.nats_url = nats_url.to_string();
        Self::connect_with_config(&cfg, allow_memory_fallback).await
    }

    /// Connect with full TLS/credential/resilience configuration from `CoreConfig`.
    pub async fn connect_with_config(
        cfg: &CoreConfig,
        allow_memory_fallback: bool,
    ) -> HelixResult<Self> {
        match NatsBus::connect(cfg).await {
            Ok(bus) => {
                info!(%cfg.nats_url, "connected to NATS");
                Ok(Self {
                    backend: BusBackend::Nats(Arc::new(bus)),
                })
            }
            Err(err) if allow_memory_fallback => {
                warn!(%cfg.nats_url, error = %err, "NATS unavailable — local memory bus");
                Ok(Self::memory())
            }
            Err(err) => Err(HelixError::unavailable(format!(
                "NATS required outside local: {err}"
            ))),
        }
    }

    /// True when connected to real NATS (not memory fallback).
    pub fn is_connected(&self) -> bool {
        matches!(self.backend, BusBackend::Nats(_))
    }

    pub fn mode(&self) -> &'static str {
        match self.backend {
            BusBackend::Nats(_) => "nats+jetstream",
            BusBackend::Memory(_) => "memory",
        }
    }

    /// JetStream available when connected to real NATS (stream HELIX_CORE ensured).
    pub fn jetstream_enabled(&self) -> bool {
        matches!(self.backend, BusBackend::Nats(_))
    }

    pub async fn publish<T: Serialize + Send + Sync>(
        &self,
        subject: &str,
        payload: &T,
    ) -> HelixResult<()> {
        let bytes = serde_json::to_vec(payload)
            .map_err(|e| HelixError::internal(format!("publish serialize: {e}")))?;
        match &self.backend {
            BusBackend::Memory(b) => b.publish(subject, Bytes::from(bytes)).await,
            BusBackend::Nats(b) => b.publish(subject, Bytes::from(bytes)).await,
        }
    }

    pub async fn subscribe(&self, subject: &str) -> HelixResult<BusSubscription> {
        match &self.backend {
            BusBackend::Memory(b) => b.subscribe(subject).await,
            BusBackend::Nats(b) => b.subscribe(subject).await,
        }
    }

    pub async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        subject: &str,
        payload: &T,
    ) -> HelixResult<R> {
        let bytes = serde_json::to_vec(payload)
            .map_err(|e| HelixError::internal(format!("request serialize: {e}")))?;
        let resp = match &self.backend {
            BusBackend::Memory(b) => b.request(subject, Bytes::from(bytes)).await?,
            BusBackend::Nats(b) => b.request(subject, Bytes::from(bytes)).await?,
        };
        serde_json::from_slice(&resp)
            .map_err(|e| HelixError::internal(format!("request deserialize: {e}")))
    }
}

pub struct BusSubscription {
    rx: broadcast::Receiver<BusMessage>,
}

impl BusSubscription {
    pub async fn next(&mut self) -> Option<BusMessage> {
        loop {
            match self.rx.recv().await {
                Ok(msg) => return Some(msg),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BusMessage {
    pub subject: String,
    pub payload: Bytes,
}

impl BusMessage {
    pub fn json<T: DeserializeOwned>(&self) -> HelixResult<T> {
        serde_json::from_slice(&self.payload)
            .map_err(|e| HelixError::validation(format!("message json: {e}")))
    }
}

struct MemoryBus {
    channels: RwLock<HashMap<String, broadcast::Sender<BusMessage>>>,
}

impl MemoryBus {
    fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    async fn sender(&self, subject: &str) -> broadcast::Sender<BusMessage> {
        let mut map = self.channels.write().await;
        map.entry(subject.to_string())
            .or_insert_with(|| broadcast::channel(1024).0)
            .clone()
    }

    async fn publish(&self, subject: &str, payload: Bytes) -> HelixResult<()> {
        let tx = self.sender(subject).await;
        let _ = tx.send(BusMessage {
            subject: subject.into(),
            payload,
        });
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> HelixResult<BusSubscription> {
        let tx = self.sender(subject).await;
        Ok(BusSubscription { rx: tx.subscribe() })
    }

    async fn request(&self, subject: &str, payload: Bytes) -> HelixResult<Bytes> {
        let _ = payload;
        let reply = serde_json::json!({
            "ok": true,
            "subject": subject,
            "mode": "memory"
        });
        Ok(Bytes::from(serde_json::to_vec(&reply).unwrap()))
    }
}

struct NatsBus {
    client: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
    js_core_fallback: bool,
}

impl NatsBus {
    async fn connect(cfg: &CoreConfig) -> HelixResult<Self> {
        let mut opts = async_nats::ConnectOptions::new()
            .name(cfg.service_name.clone())
            .require_tls(cfg.nats_require_tls)
            .connection_timeout(Duration::from_secs(cfg.nats_connection_timeout_secs))
            .ping_interval(Duration::from_secs(24));

        if cfg.nats_tls_first {
            opts = opts.tls_first();
        }
        if cfg.nats_retry_on_initial_connect {
            opts = opts.retry_on_initial_connect();
        }
        if cfg.nats_max_reconnects > 0 {
            opts = opts.max_reconnects(cfg.nats_max_reconnects as usize);
        }

        if let Some(path) = cfg.nats_creds_file.as_deref() {
            opts = opts
                .credentials_file(PathBuf::from(path))
                .await
                .map_err(|e| HelixError::validation(format!("nats creds file: {e}")))?;
        } else if cfg.nats_jwt.is_some() && cfg.nats_nkey.is_some() {
            let jwt = cfg.nats_jwt.clone().unwrap();
            let nkey = cfg.nats_nkey.clone().unwrap();
            let creds = format!(
                "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n************************* IMPORTANT *************************\nNKEY Seed printed below can be used sign and prove identity.\nNKEYs are sensitive and should be treated as secrets.\n\n-----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n",
                jwt, nkey
            );
            opts = opts
                .credentials(&creds)
                .map_err(|e| HelixError::validation(format!("nats jwt/nkey: {e}")))?;
        }

        if let Some(path) = cfg.nats_tls_ca_file.as_deref() {
            opts = opts.add_root_certificates(PathBuf::from(path));
        }
        if let (Some(cert), Some(key)) = (
            cfg.nats_tls_cert_file.as_deref(),
            cfg.nats_tls_key_file.as_deref(),
        ) {
            opts = opts.add_client_certificate(PathBuf::from(cert), PathBuf::from(key));
        }

        let client = opts
            .connect(&cfg.nats_url)
            .await
            .map_err(|e| HelixError::dependency(format!("nats connect: {e}")))?;
        let jetstream = async_nats::jetstream::new(client.clone());

        // Ensure core durable stream exists with bounded retries. Outside local this keeps
        // the service fail-closed if JetStream is not ready, while tolerating brief NATS
        // startup windows.
        let stream_cfg = async_nats::jetstream::stream::Config {
            name: "HELIX_CORE".into(),
            subjects: vec!["helix.>".into()],
            max_messages: 1_000_000,
            retention: async_nats::jetstream::stream::RetentionPolicy::Limits,
            ..Default::default()
        };

        let mut last_err = None;
        let attempts = cfg.nats_js_retry_attempts.max(1);
        for attempt in 1..=attempts {
            match jetstream.get_or_create_stream(stream_cfg.clone()).await {
                Ok(_) => break,
                Err(e) => {
                    warn!(%attempt, %attempts, error = %e, "jetstream stream setup failed");
                    last_err = Some(e);
                    if attempt < attempts {
                        sleep(Duration::from_millis(cfg.nats_js_retry_backoff_ms)).await;
                    }
                }
            }
        }
        if let Some(e) = last_err {
            return Err(HelixError::dependency(format!(
                "jetstream stream create: {e}"
            )));
        }

        Ok(Self {
            client,
            jetstream,
            js_core_fallback: cfg.nats_js_core_fallback,
        })
    }

    async fn publish(&self, subject: &str, payload: Bytes) -> HelixResult<()> {
        // Prefer JetStream durable publish; fall back to core NATS only when explicitly allowed.
        match self
            .jetstream
            .publish(subject.to_string(), payload.clone())
            .await
        {
            Ok(ack) => {
                ack.await
                    .map_err(|e| HelixError::dependency(format!("jetstream publish ack: {e}")))?;
                Ok(())
            }
            Err(js_err) => {
                if self.js_core_fallback {
                    tracing::warn!(error = %js_err, subject, "jetstream publish failed — falling back to core nats");
                    self.client
                        .publish(subject.to_string(), payload)
                        .await
                        .map_err(|e| HelixError::dependency(format!("nats publish: {e}")))?;
                    Ok(())
                } else {
                    Err(HelixError::dependency(format!(
                        "jetstream publish: {js_err}"
                    )))
                }
            }
        }
    }

    async fn subscribe(&self, subject: &str) -> HelixResult<BusSubscription> {
        let mut sub = self
            .client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| HelixError::dependency(format!("nats subscribe: {e}")))?;

        let (tx, rx) = broadcast::channel(1024);
        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                let _ = tx.send(BusMessage {
                    subject: msg.subject.to_string(),
                    payload: msg.payload,
                });
            }
        });
        Ok(BusSubscription { rx })
    }

    async fn request(&self, subject: &str, payload: Bytes) -> HelixResult<Bytes> {
        let msg = self
            .client
            .request(subject.to_string(), payload)
            .await
            .map_err(|e| HelixError::dependency(format!("nats request: {e}")))?;
        Ok(msg.payload)
    }
}

pub fn subject(product_prefix: &str, domain: &str, event: &str) -> String {
    format!("{product_prefix}.{domain}.{event}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn memory_publish_subscribe() {
        let bus = HelixBus::memory();
        let mut sub = bus.subscribe("helix.test.ping").await.unwrap();
        bus.publish("helix.test.ping", &json!({"n": 1}))
            .await
            .unwrap();
        let msg = sub.next().await.expect("message");
        let v: serde_json::Value = msg.json().unwrap();
        assert_eq!(v["n"], 1);
    }

    #[tokio::test]
    async fn connect_with_policy_fail_closed() {
        // Bad URL + no fallback → error (not silent memory).
        let err = HelixBus::connect_with_policy("nats://127.0.0.1:1", false).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn connect_with_policy_allows_memory_local() {
        // The legacy path still loads CoreConfig; opt-in to local defaults for this test.
        std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
        let bus = HelixBus::connect_with_policy("nats://127.0.0.1:1", true)
            .await
            .unwrap();
        assert_eq!(bus.mode(), "memory");
        assert!(!bus.is_connected());
    }
}
