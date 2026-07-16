//! Minimal S3-compatible client for MinIO (path-style, SigV4).

use chrono::Utc;
use hmac::{Hmac, Mac};
use quick_xml::events::Event;
use quick_xml::Reader;
use sha2::{Digest, Sha256};
use shared_core::{HelixError, HelixResult};
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct ObjectStoreConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

impl ObjectStoreConfig {
    /// Build config from explicit values. Secrets must be supplied by the caller
    /// (typically `shared_core::CoreConfig`); this crate no longer embeds defaults.
    pub fn new(
        endpoint: impl Into<String>,
        bucket: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            bucket: bucket.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region: region.into(),
        }
    }
}

#[derive(Clone)]
pub struct ObjectStore {
    cfg: ObjectStoreConfig,
    http: reqwest::Client,
}

impl ObjectStore {
    pub fn new(cfg: ObjectStoreConfig) -> HelixResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| HelixError::internal(format!("object store http: {e}")))?;
        Ok(Self { cfg, http })
    }

    pub fn from_config(cfg: &ObjectStoreConfig) -> HelixResult<Self> {
        Self::new(cfg.clone())
    }

    pub fn config(&self) -> &ObjectStoreConfig {
        &self.cfg
    }

    /// Live check: GET /minio/health/live (short timeout for /healthz).
    pub async fn health(&self) -> HelixResult<bool> {
        let base = self.cfg.endpoint.trim_end_matches('/');
        let url = format!("{base}/minio/health/live");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(800))
            .build()
            .map_err(|e| HelixError::internal(format!("minio health http: {e}")))?;
        match client.get(&url).send().await {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Startup verification: endpoint reachable and bucket exists.
    /// Returns a dependency error if the bucket is missing or unreachable.
    pub async fn verify(&self) -> HelixResult<()> {
        let base = self.cfg.endpoint.trim_end_matches('/');
        // HEAD the bucket root; MinIO returns 200/403 if it exists, 404 if not.
        let url = format!("{base}/{}/", self.cfg.bucket);
        let resp = self
            .http
            .head(&url)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("minio verify: {e}")))?;
        if resp.status().as_u16() == 404 {
            return Err(HelixError::dependency(format!(
                "minio bucket '{}' not found",
                self.cfg.bucket
            )));
        }
        if !resp.status().is_success() && resp.status().as_u16() != 403 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(HelixError::dependency(format!(
                "minio verify {status}: {body}"
            )));
        }
        Ok(())
    }

    pub async fn put_object(&self, key: &str, bytes: &[u8], content_type: &str) -> HelixResult<()> {
        let key = key.trim_start_matches('/');
        let host = host_header(&self.cfg.endpoint)?;
        let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = &amz_date[..8];
        let payload_hash = hex::encode(Sha256::digest(bytes));
        let canonical_uri = format!("/{}/{}", self.cfg.bucket, key);
        let canonical_headers = format!(
            "content-type:{content_type}\nhost:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
        );
        let signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date";
        let canonical_request = format!(
            "PUT\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let auth = sign_v4(
            &self.cfg,
            "PUT",
            &canonical_request,
            &amz_date,
            date_stamp,
            signed_headers,
        )?;

        let url = format!(
            "{}/{}/{}",
            self.cfg.endpoint.trim_end_matches('/'),
            self.cfg.bucket,
            key
        );
        let resp = self
            .http
            .put(&url)
            .header("Host", host)
            .header("Content-Type", content_type)
            .header("x-amz-content-sha256", &payload_hash)
            .header("x-amz-date", &amz_date)
            .header("Authorization", auth)
            .body(bytes.to_vec())
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("minio put: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(HelixError::dependency(format!(
                "minio put {status}: {body}"
            )));
        }
        Ok(())
    }

    pub async fn get_object(&self, key: &str) -> HelixResult<Vec<u8>> {
        let key = key.trim_start_matches('/');
        let host = host_header(&self.cfg.endpoint)?;
        let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = &amz_date[..8];
        let payload_hash = hex::encode(Sha256::digest([]));
        let canonical_uri = format!("/{}/{}", self.cfg.bucket, key);
        let canonical_headers =
            format!("host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n");
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
        let canonical_request = format!(
            "GET\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let auth = sign_v4(
            &self.cfg,
            "GET",
            &canonical_request,
            &amz_date,
            date_stamp,
            signed_headers,
        )?;

        let url = format!(
            "{}/{}/{}",
            self.cfg.endpoint.trim_end_matches('/'),
            self.cfg.bucket,
            key
        );
        let resp = self
            .http
            .get(&url)
            .header("Host", host)
            .header("x-amz-content-sha256", &payload_hash)
            .header("x-amz-date", &amz_date)
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("minio get: {e}")))?;
        if resp.status().as_u16() == 404 {
            return Err(HelixError::not_found(format!("object {key}")));
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(HelixError::dependency(format!(
                "minio get {status}: {body}"
            )));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| HelixError::dependency(format!("minio get body: {e}")))
    }

    pub async fn delete_object(&self, key: &str) -> HelixResult<()> {
        let key = key.trim_start_matches('/');
        let host = host_header(&self.cfg.endpoint)?;
        let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = &amz_date[..8];
        let payload_hash = hex::encode(Sha256::digest([]));
        let canonical_uri = format!("/{}/{}", self.cfg.bucket, key);
        let canonical_headers =
            format!("host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n");
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
        let canonical_request = format!(
            "DELETE\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let auth = sign_v4(
            &self.cfg,
            "DELETE",
            &canonical_request,
            &amz_date,
            date_stamp,
            signed_headers,
        )?;

        let url = format!(
            "{}/{}/{}",
            self.cfg.endpoint.trim_end_matches('/'),
            self.cfg.bucket,
            key
        );
        let resp = self
            .http
            .delete(&url)
            .header("Host", host)
            .header("x-amz-content-sha256", &payload_hash)
            .header("x-amz-date", &amz_date)
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("minio delete: {e}")))?;
        if resp.status().as_u16() == 404 {
            return Ok(());
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(HelixError::dependency(format!(
                "minio delete {status}: {body}"
            )));
        }
        Ok(())
    }

    /// List object keys under a prefix using S3 ListObjectsV2 (SigV4 signed).
    /// Handles pagination up to the bucket's configured max-keys limit.
    pub async fn list_keys(&self, prefix: &str) -> HelixResult<Vec<String>> {
        let base = self.cfg.endpoint.trim_end_matches('/');
        let host = host_header(&self.cfg.endpoint)?;
        let encoded_prefix = percent_encode(prefix);
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;
        let mut iterations = 0u32;
        const MAX_ITERATIONS: u32 = 100;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(HelixError::dependency(
                    "minio list_keys: too many pagination iterations",
                ));
            }

            let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
            let date_stamp = &amz_date[..8];
            let payload_hash = hex::encode(Sha256::digest([]));

            let mut query_parts: Vec<(&str, String)> = vec![
                ("list-type", "2".into()),
                ("max-keys", "1000".into()),
                ("prefix", encoded_prefix.clone()),
            ];
            if let Some(ref token) = continuation_token {
                query_parts.push(("continuation-token", percent_encode(token)));
            }
            // Canonical query string must be sorted by key.
            query_parts.sort_by(|a, b| a.0.cmp(b.0));
            let canonical_query = query_parts
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            let url_query = canonical_query.clone();

            let canonical_uri = format!("/{}", self.cfg.bucket);
            let canonical_headers = format!(
                "host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
            );
            let signed_headers = "host;x-amz-content-sha256;x-amz-date";
            let canonical_request = format!(
                "GET\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
            );
            let auth = sign_v4(
                &self.cfg,
                "GET",
                &canonical_request,
                &amz_date,
                date_stamp,
                signed_headers,
            )?;

            let url = format!("{base}/{canonical_uri}?{url_query}");
            let resp = self
                .http
                .get(&url)
                .header("Host", &host)
                .header("x-amz-content-sha256", &payload_hash)
                .header("x-amz-date", &amz_date)
                .header("Authorization", auth)
                .send()
                .await
                .map_err(|e| HelixError::dependency(format!("minio list: {e}")))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(HelixError::dependency(format!(
                    "minio list {status}: {body}"
                )));
            }
            let body = resp
                .text()
                .await
                .map_err(|e| HelixError::dependency(format!("minio list body: {e}")))?;
            let (mut page_keys, truncated, next_token) = parse_list_response(&body)?;
            keys.append(&mut page_keys);
            if !truncated {
                break;
            }
            let Some(token) = next_token else {
                break;
            };
            continuation_token = Some(token);
        }

        Ok(keys)
    }
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn parse_list_response(body: &str) -> HelixResult<(Vec<String>, bool, Option<String>)> {
    let mut reader = Reader::from_str(body);
    reader.config_mut().trim_text(true);
    let mut keys = Vec::new();
    let mut truncated = false;
    let mut next_continuation_token: Option<String> = None;
    let mut current_element: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                current_element = Some(name);
            }
            Ok(Event::Text(e)) => {
                let text = e
                    .unescape()
                    .map_err(|err| HelixError::internal(format!("minio list xml: {err}")))?
                    .into_owned();
                if let Some(ref name) = current_element {
                    match name.as_str() {
                        "Key" => keys.push(text),
                        "IsTruncated" => truncated = text.eq_ignore_ascii_case("true"),
                        "NextContinuationToken" => next_continuation_token = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_element = None;
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(HelixError::internal(format!("minio list xml: {err}")));
            }
            _ => {}
        }
    }

    Ok((keys, truncated, next_continuation_token))
}

fn host_header(endpoint: &str) -> HelixResult<String> {
    let u = endpoint
        .trim_end_matches('/')
        .strip_prefix("https://")
        .or_else(|| endpoint.trim_end_matches('/').strip_prefix("http://"))
        .unwrap_or(endpoint.trim_end_matches('/'));
    Ok(u.to_string())
}

fn sign_v4(
    cfg: &ObjectStoreConfig,
    _method: &str,
    canonical_request: &str,
    amz_date: &str,
    date_stamp: &str,
    signed_headers: &str,
) -> HelixResult<String> {
    let algorithm = "AWS4-HMAC-SHA256";
    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, cfg.region);
    let canonical_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));
    let string_to_sign = format!("{algorithm}\n{amz_date}\n{credential_scope}\n{canonical_hash}");

    let signing_key = signing_key(&cfg.secret_key, date_stamp, &cfg.region)?;
    let mut mac = HmacSha256::new_from_slice(&signing_key)
        .map_err(|e| HelixError::internal(format!("hmac: {e}")))?;
    mac.update(string_to_sign.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    Ok(format!(
        "{algorithm} Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
        cfg.access_key
    ))
}

fn signing_key(secret: &str, date_stamp: &str, region: &str) -> HelixResult<Vec<u8>> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date_stamp.as_bytes())?;
    let k_region = hmac_sha256(&k_date, region.as_bytes())?;
    let k_service = hmac_sha256(&k_region, b"s3")?;
    hmac_sha256(&k_service, b"aws4_request")
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> HelixResult<Vec<u8>> {
    let mut mac =
        HmacSha256::new_from_slice(key).map_err(|e| HelixError::internal(format!("hmac: {e}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}
