//! Webhook URL policy — SSRF hardening for outbound deliveries.
//!
//! Private/loopback allowed only when:
//! - `HELIX_CODE_WEBHOOK_ALLOW_PRIVATE=1`, or
//! - `HELIX_ENV=local` / `dev`
//!
//! **Not** tied to `HELIX_ALLOW_DEV_HEADERS` (decoupled per Kimi residual).
//!
//! Outside local/dev: HTTPS required; `HELIX_CODE_WEBHOOK_ALLOW_HOSTS` **required**
//! (fail-closed when empty). Local may omit allowlist for smoke.

use shared_core::{HelixError, HelixResult};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use url::Url;

pub fn is_local_env() -> bool {
    std::env::var("HELIX_ENV")
        .map(|v| v.eq_ignore_ascii_case("local") || v.eq_ignore_ascii_case("dev"))
        .unwrap_or(false)
}

pub fn allow_private_webhook_targets() -> bool {
    env_truthy("HELIX_CODE_WEBHOOK_ALLOW_PRIVATE") || is_local_env()
}

/// Outside local/dev, only https is accepted (unless HELIX_CODE_WEBHOOK_ALLOW_HTTP=1).
pub fn https_required() -> bool {
    if env_truthy("HELIX_CODE_WEBHOOK_ALLOW_HTTP") {
        return false;
    }
    !is_local_env()
}

fn env_truthy(k: &str) -> bool {
    std::env::var(k)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Optional comma-separated host allowlist (exact or leading-dot suffix match).
fn host_allowlist() -> Vec<String> {
    std::env::var("HELIX_CODE_WEBHOOK_ALLOW_HOSTS")
        .ok()
        .map(|s| {
            s.split(',')
                .map(|p| p.trim().to_ascii_lowercase())
                .filter(|p| !p.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn host_on_allowlist(host: &str) -> bool {
    let list = host_allowlist();
    if list.is_empty() {
        // Fail closed outside local — must configure HELIX_CODE_WEBHOOK_ALLOW_HOSTS
        return is_local_env();
    }
    list.iter().any(|entry| {
        if let Some(suffix) = entry.strip_prefix('.') {
            host == suffix || host.ends_with(entry.as_str()) || host.ends_with(suffix)
        } else {
            host == entry.as_str()
        }
    })
}

/// Validate webhook URL at create time and before delivery.
pub fn validate_webhook_url(raw: &str) -> HelixResult<()> {
    let (_url, _ips) = parse_and_resolve(raw)?;
    Ok(())
}

/// Parse, validate scheme/host/IP policy, and resolve allowed IPs.
pub fn parse_and_resolve(raw: &str) -> HelixResult<(Url, Vec<IpAddr>)> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(HelixError::validation("webhook url required"));
    }
    if raw.len() > 2048 {
        return Err(HelixError::validation("webhook url too long"));
    }
    let url =
        Url::parse(raw).map_err(|e| HelixError::validation(format!("invalid webhook url: {e}")))?;
    match url.scheme() {
        "https" => {}
        "http" => {
            if https_required() {
                return Err(HelixError::validation(
                    "webhook must use https outside local (set HELIX_CODE_WEBHOOK_ALLOW_HTTP=1 to override)",
                ));
            }
        }
        other => {
            return Err(HelixError::validation(format!(
                "webhook scheme '{other}' not allowed (http/https only)"
            )));
        }
    }
    let host = url
        .host_str()
        .ok_or_else(|| HelixError::validation("webhook url missing host"))?
        .to_ascii_lowercase();

    if is_blocked_hostname(&host) {
        return Err(HelixError::validation(format!(
            "webhook host '{host}' blocked (SSRF policy)"
        )));
    }
    if !host_on_allowlist(&host) {
        return Err(HelixError::validation(format!(
            "webhook host '{host}' not on HELIX_CODE_WEBHOOK_ALLOW_HOSTS (required outside local)"
        )));
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        if !ip_allowed(ip) {
            return Err(HelixError::validation(format!(
                "webhook IP {ip} blocked (SSRF policy; set HELIX_CODE_WEBHOOK_ALLOW_PRIVATE=1 for private)"
            )));
        }
        return Ok((url, vec![ip]));
    }

    let port = url.port_or_known_default().unwrap_or(443);
    let mut ips = Vec::new();
    match (host.as_str(), port).to_socket_addrs() {
        Ok(addrs) => {
            for sa in addrs {
                let ip = sa.ip();
                if !ip_allowed(ip) {
                    return Err(HelixError::validation(format!(
                        "webhook host '{host}' resolves to blocked IP {ip} (SSRF policy)"
                    )));
                }
                if !ips.contains(&ip) {
                    ips.push(ip);
                }
            }
        }
        Err(e) => {
            if is_local_env() {
                return Ok((url, ips));
            }
            return Err(HelixError::validation(format!(
                "webhook host '{host}' DNS resolve failed: {e}"
            )));
        }
    }
    if ips.is_empty() && !is_local_env() {
        return Err(HelixError::validation(format!(
            "webhook host '{host}' resolved to no addresses"
        )));
    }
    Ok((url, ips))
}

/// Build a pin-connect URL (http://IP/path) + Host header value for delivery.
pub fn pinned_request_target(url: &Url, ip: IpAddr) -> HelixResult<(String, String, u16)> {
    let host = url
        .host_str()
        .ok_or_else(|| HelixError::validation("missing host"))?
        .to_string();
    let port = url
        .port_or_known_default()
        .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
    let path = if url.path().is_empty() {
        "/"
    } else {
        url.path()
    };
    let query = url.query().map(|q| format!("?{q}")).unwrap_or_default();
    let host_header =
        if (url.scheme() == "https" && port == 443) || (url.scheme() == "http" && port == 80) {
            host.clone()
        } else {
            format!("{host}:{port}")
        };
    let ip_lit = match ip {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => format!("[{v6}]"),
    };
    let target = format!("{}://{}:{}{}{}", url.scheme(), ip_lit, port, path, query);
    Ok((target, host_header, port))
}

fn is_blocked_hostname(host: &str) -> bool {
    const EXACT: &[&str] = &[
        "metadata.google.internal",
        "metadata",
        "metadata.internal",
        "kubernetes.default",
        "kubernetes.default.svc",
        "kubernetes.default.svc.cluster.local",
    ];
    if EXACT.contains(&host) {
        return true;
    }
    if host.ends_with(".internal")
        || host.ends_with(".localdomain")
        || (host.contains("metadata")
            && (host.contains("google") || host.contains("aws") || host.contains("azure")))
    {
        return true;
    }
    if !allow_private_webhook_targets()
        && (host == "localhost"
            || host.ends_with(".localhost")
            || host.ends_with(".local")
            || host.ends_with(".lan")
            || host.ends_with(".home")
            || host.ends_with(".corp"))
    {
        return true;
    }
    false
}

fn ip_allowed(ip: IpAddr) -> bool {
    if is_metadata_or_dangerous(ip) {
        return false;
    }
    if is_private_or_loopback(ip) {
        return allow_private_webhook_targets();
    }
    true
}

fn is_metadata_or_dangerous(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4 == Ipv4Addr::new(169, 254, 169, 254)
                || v4 == Ipv4Addr::new(169, 254, 170, 2)
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_metadata_or_dangerous(IpAddr::V4(v4));
            }
            false
        }
    }
}

fn is_private_or_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 64)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unique_local()
                || is_ipv6_link_local(v6)
                || v6.is_unspecified()
        }
    }
}

fn is_ipv6_link_local(v6: Ipv6Addr) -> bool {
    let s = v6.segments();
    (s[0] & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn blocks_metadata_ip() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("HELIX_CODE_WEBHOOK_ALLOW_PRIVATE");
        std::env::set_var("HELIX_ENV", "production");
        std::env::remove_var("HELIX_CODE_WEBHOOK_ALLOW_HTTP");
        std::env::set_var("HELIX_CODE_WEBHOOK_ALLOW_HOSTS", "example.com");
        assert!(validate_webhook_url("http://169.254.169.254/latest/meta-data").is_err());
        assert!(validate_webhook_url("https://metadata.google.internal/").is_err());
        std::env::remove_var("HELIX_ENV");
        std::env::remove_var("HELIX_CODE_WEBHOOK_ALLOW_HOSTS");
    }

    #[test]
    fn private_requires_webhook_flag_not_dev_headers() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("HELIX_CODE_WEBHOOK_ALLOW_PRIVATE");
        std::env::remove_var("HELIX_ENV");
        std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
        // dev headers alone must NOT open private webhook targets
        assert!(validate_webhook_url("http://127.0.0.1:9/hook").is_err());
        std::env::set_var("HELIX_ENV", "local");
        assert!(validate_webhook_url("http://127.0.0.1:9/hook").is_ok());
        std::env::remove_var("HELIX_ALLOW_DEV_HEADERS");
        std::env::remove_var("HELIX_ENV");
    }

    #[test]
    fn production_requires_host_allowlist() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("HELIX_ENV", "production");
        std::env::remove_var("HELIX_CODE_WEBHOOK_ALLOW_HOSTS");
        assert!(validate_webhook_url("https://hooks.example.com/x").is_err());
        std::env::set_var("HELIX_CODE_WEBHOOK_ALLOW_HOSTS", "hooks.example.com");
        // may fail DNS offline — allow either ok or resolve error, not allowlist miss
        let r = validate_webhook_url("https://hooks.example.com/x");
        let msg = format!("{r:?}");
        assert!(
            r.is_ok() || msg.contains("DNS") || msg.contains("resolve") || msg.contains("address"),
            "{msg}"
        );
        std::env::remove_var("HELIX_ENV");
        std::env::remove_var("HELIX_CODE_WEBHOOK_ALLOW_HOSTS");
    }

    #[test]
    fn pin_target_ipv4() {
        let url = Url::parse("https://hooks.example.com/a?b=1").unwrap();
        let (t, host, _) =
            pinned_request_target(&url, IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))).unwrap();
        assert_eq!(t, "https://1.2.3.4:443/a?b=1");
        assert_eq!(host, "hooks.example.com");
    }
}
