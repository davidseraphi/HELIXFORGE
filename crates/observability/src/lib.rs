//! Observability bootstrap — structured logging, OTLP/HTTP span export, metrics.

use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

static OTLP_ACTIVE: AtomicBool = AtomicBool::new(false);
static OTLP_ENDPOINT: Mutex<Option<String>> = Mutex::new(None);
static OTLP_SAMPLE_RATE: Mutex<f64> = Mutex::new(1.0);

#[derive(Clone, Default)]
pub struct MetricsRegistry {
    counters: Arc<Mutex<HashMap<String, u64>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc(&self, name: &str, by: u64) {
        let mut guard = self.counters.lock();
        *guard.entry(name.to_string()).or_insert(0) += by;
    }

    pub fn get(&self, name: &str) -> u64 {
        *self.counters.lock().get(name).unwrap_or(&0)
    }

    pub fn snapshot(&self) -> HashMap<String, u64> {
        self.counters.lock().clone()
    }
}

/// Initialize tracing without OTLP.
pub fn init_tracing(service_name: &str, json: bool) {
    init_tracing_with_otlp(service_name, json, None, 1.0);
}

/// Initialize tracing. When `otlp_endpoint` is set, enable OTLP/HTTP JSON export
/// (works with Jaeger all-in-one `COLLECTOR_OTLP_ENABLED` on :4318).
pub fn init_tracing_with_otlp(
    service_name: &str,
    json: bool,
    otlp_endpoint: Option<&str>,
    sample_rate: f64,
) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,sqlx=warn"));

    if json {
        let _ = fmt::Subscriber::builder()
            .with_env_filter(filter)
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .try_init();
    } else {
        let _ = fmt::Subscriber::builder()
            .with_env_filter(filter)
            .with_ansi(true)
            .try_init();
    }

    let rate = sample_rate.clamp(0.0, 1.0);
    *OTLP_SAMPLE_RATE.lock() = rate;

    if let Some(ep) = otlp_endpoint {
        // Prefer HTTP OTLP path (4318). If user passed gRPC 4317, map to 4318 for JSON export.
        let http_ep = normalize_otlp_http(ep);
        *OTLP_ENDPOINT.lock() = Some(http_ep.clone());
        OTLP_ACTIVE.store(true, Ordering::SeqCst);
        tracing::info!(
            %service_name,
            endpoint = %http_ep,
            sample_rate = %rate,
            "observability + OTLP/HTTP export enabled"
        );
        // Emit a bootstrap span so collectors show the service immediately.
        let svc = service_name.to_string();
        let ep2 = http_ep;
        tokio::spawn(async move {
            let _ = export_span(&ep2, &svc, "helix.boot", 0, true, None, None).await;
        });
    } else {
        tracing::info!(%service_name, "observability initialized (fmt only)");
    }
}

fn normalize_otlp_http(endpoint: &str) -> String {
    let e = endpoint.trim_end_matches('/');
    if e.ends_with(":4317") {
        format!("{}:4318", e.trim_end_matches(":4317"))
    } else {
        e.to_string()
    }
}

pub fn otlp_enabled() -> bool {
    OTLP_ACTIVE.load(Ordering::SeqCst)
}

pub fn otlp_endpoint() -> Option<String> {
    OTLP_ENDPOINT.lock().clone()
}

pub fn otlp_sample_rate() -> f64 {
    *OTLP_SAMPLE_RATE.lock()
}

/// Export a single span via OTLP/HTTP JSON (Jaeger/OTel collector compatible).
///
/// This is an intentionally lightweight *probe*: it supports W3C traceparent
/// propagation and deterministic sampling, but it is not a full OpenTelemetry SDK.
/// For production tail-sampling or baggage, migrate to `opentelemetry-otlp`.
pub async fn export_span(
    endpoint: &str,
    service_name: &str,
    name: &str,
    duration_ms: u64,
    ok: bool,
    trace_id: Option<&str>,
    parent_span_id: Option<&str>,
) -> Result<(), String> {
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let start = now_ns.saturating_sub((duration_ms as u128) * 1_000_000);

    // Honour propagated trace context; otherwise start a fresh trace.
    let trace_id = trace_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{:032x}", rand_u128()));

    // Deterministic sampling: the same trace_id makes the same decision everywhere.
    if !sample_decision(otlp_sample_rate(), &trace_id) {
        return Ok(());
    }

    let span_id = format!("{:016x}", rand_u64());
    let status_code = if ok { 1 } else { 2 }; // OTEL: 1=Ok 2=Error

    let mut span = serde_json::json!({
        "traceId": trace_id,
        "spanId": span_id,
        "name": name,
        "kind": 2,
        "startTimeUnixNano": start.to_string(),
        "endTimeUnixNano": now_ns.to_string(),
        "status": {"code": status_code}
    });
    if let Some(parent) = parent_span_id.filter(|s| !s.is_empty()) {
        span["parentSpanId"] = serde_json::json!(parent);
    }

    let body = serde_json::json!({
        "resourceSpans": [{
            "resource": {
                "attributes": [
                    {"key": "service.name", "value": {"stringValue": service_name}}
                ]
            },
            "scopeSpans": [{
                "scope": {"name": "helixforge", "version": env!("CARGO_PKG_VERSION")},
                "spans": [span]
            }]
        }]
    });

    let url = if endpoint.contains("/v1/traces") {
        endpoint.to_string()
    } else {
        format!("{}/v1/traces", endpoint.trim_end_matches('/'))
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("otlp status {}", resp.status()));
    }
    Ok(())
}

fn sample_decision(rate: f64, trace_id: &str) -> bool {
    let rate = rate.clamp(0.0, 1.0);
    if rate >= 1.0 {
        return true;
    }
    if rate <= 0.0 {
        return false;
    }
    let digits: String = trace_id
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(16)
        .collect();
    let val = u64::from_str_radix(&digits, 16).unwrap_or(0);
    let threshold = (rate * (u64::MAX as f64)) as u64;
    val <= threshold
}

fn parse_traceparent(header: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = header.split('-').collect();
    if parts.len() != 4 || parts[0] != "00" {
        return None;
    }
    let trace_id = parts[1].to_lowercase();
    let parent_id = parts[2].to_lowercase();
    if trace_id.len() != 32
        || parent_id.len() != 16
        || !trace_id.chars().all(|c| c.is_ascii_hexdigit())
        || !parent_id.chars().all(|c| c.is_ascii_hexdigit())
    {
        return None;
    }
    Some((trace_id, parent_id))
}

/// Export with bounded retries (sovereign observability depth).
/// `traceparent` is an optional W3C `traceparent` header value for context propagation.
pub fn emit_span(
    service_name: &str,
    name: &str,
    duration_ms: u64,
    ok: bool,
    traceparent: Option<&str>,
) {
    if !otlp_enabled() {
        return;
    }
    let Some(ep) = otlp_endpoint() else {
        return;
    };
    let (trace_id, parent_span_id) = traceparent
        .and_then(parse_traceparent)
        .map(|(t, p)| (Some(t), Some(p)))
        .unwrap_or((None, None));
    let svc = service_name.to_string();
    let n = name.to_string();
    tokio::spawn(async move {
        let mut last_err = None;
        for attempt in 0..3u32 {
            match export_span(
                &ep,
                &svc,
                &n,
                duration_ms,
                ok,
                trace_id.as_deref(),
                parent_span_id.as_deref(),
            )
            .await
            {
                Ok(()) => return,
                Err(e) => {
                    last_err = Some(e);
                    tokio::time::sleep(std::time::Duration::from_millis(50 * (1 << attempt))).await;
                }
            }
        }
        if let Some(e) = last_err {
            tracing::debug!(error = %e, "otlp export failed after retries");
        }
    });
}

fn rand_u64() -> u64 {
    let mut buf = [0u8; 8];
    if getrandom::getrandom(&mut buf).is_ok() {
        return u64::from_le_bytes(buf);
    }
    // Fallback only if OS RNG unavailable.
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::Instant::now().hash(&mut h);
    std::thread::current().id().hash(&mut h);
    h.finish()
}

fn rand_u128() -> u128 {
    let a = rand_u64() as u128;
    let b = rand_u64() as u128;
    (a << 64) | b
}

pub fn init_otlp_hint(endpoint: &str) {
    tracing::info!(endpoint = %endpoint, "OTEL_EXPORTER_OTLP_ENDPOINT configured");
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub service: String,
    pub status: &'static str,
    pub version: &'static str,
    pub checks: HashMap<String, CheckResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub ok: bool,
    pub detail: String,
}

impl HealthStatus {
    pub fn healthy(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
            checks: HashMap::new(),
        }
    }

    pub fn with_check(
        mut self,
        name: impl Into<String>,
        ok: bool,
        detail: impl Into<String>,
    ) -> Self {
        self.checks.insert(
            name.into(),
            CheckResult {
                ok,
                detail: detail.into(),
            },
        );
        if self.checks.values().any(|c| !c.ok) {
            self.status = "degraded";
        }
        self
    }
}
