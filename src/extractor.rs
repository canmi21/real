/* src/extractor.rs */

use std::collections::HashMap;
use std::net::IpAddr;

/// Type alias for header maps. Can be any map-like structure with string keys and values.
pub type HeaderMap = HashMap<String, String>;

/// Configuration for IP extraction behavior.
#[derive(Debug, Clone)]
pub struct IpExtractor {
    /// Headers to check for real IP, in order of preference.
    pub headers: Vec<String>,
    /// Whether to trust private IP addresses from headers.
    pub trust_private_ips: bool,
    /// Whether to use the first IP in X-Forwarded-For chain.
    pub use_first_forwarded: bool,
}

impl Default for IpExtractor {
    fn default() -> Self {
        Self {
            headers: vec![
                "x-real-ip".to_string(),
                "cf-connecting-ip".to_string(),
                "x-forwarded-for".to_string(),
                "x-forwarded".to_string(),
                "forwarded-for".to_string(),
                "forwarded".to_string(),
            ],
            trust_private_ips: false,
            use_first_forwarded: true,
        }
    }
}

impl IpExtractor {
    /// Create a new IP extractor with custom configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set headers to check for real IP.
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set whether to trust private IP addresses from headers.
    pub fn trust_private_ips(mut self, trust: bool) -> Self {
        self.trust_private_ips = trust;
        self
    }

    /// Set whether to use the first IP in X-Forwarded-For chain.
    pub fn use_first_forwarded(mut self, use_first: bool) -> Self {
        self.use_first_forwarded = use_first;
        self
    }

    /// Extract the real IP address from headers with fallback.
    pub fn extract(&self, headers: &HeaderMap, fallback_ip: Option<String>) -> Option<IpAddr> {
        // Try to extract from headers first
        if let Some(ip) = self.extract_from_headers(headers) {
            return Some(ip);
        }

        // Fallback to provided IP
        if let Some(fallback) = fallback_ip {
            if let Ok(ip) = fallback.parse::<IpAddr>() {
                return Some(ip);
            }
        }

        None
    }

    /// Extract IP from headers only.
    fn extract_from_headers(&self, headers: &HeaderMap) -> Option<IpAddr> {
        for header_name in &self.headers {
            if let Some(header_value) = headers.get(&header_name.to_lowercase()) {
                if let Some(ip) = self.parse_header_value(header_value) {
                    if self.is_valid_ip(&ip) {
                        return Some(ip);
                    }
                }
            }
        }
        None
    }

    /// Parse header value and extract IP address.
    fn parse_header_value(&self, value: &str) -> Option<IpAddr> {
        let value = value.trim();

        // Handle X-Forwarded-For format: "client, proxy1, proxy2"
        if value.contains(',') {
            let ips: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
            let ip_iter: Box<dyn Iterator<Item = &&str>> = if self.use_first_forwarded {
                Box::new(ips.iter())
            } else {
                Box::new(ips.iter().rev())
            };

            for ip_str in ip_iter {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        } else {
            // Single IP address
            if let Ok(ip) = value.parse::<IpAddr>() {
                return Some(ip);
            }
        }

        None
    }

    /// Check if IP is valid based on configuration.
    fn is_valid_ip(&self, ip: &IpAddr) -> bool {
        if !self.trust_private_ips && self.is_private_ip(ip) {
            return false;
        }
        true
    }

    /// Check if IP is private/internal.
    fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local(),
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() ||
                (ipv6.segments()[0] & 0xfe00) == 0xfc00 || // Unique local
                (ipv6.segments()[0] & 0xffc0) == 0xfe80 // Link local
            }
        }
    }
}

/// Convenience function to extract real IP with default configuration that trusts private IPs.
///
/// This function is a shortcut for `IpExtractor::default().trust_private_ips(true)`.
///
/// # Arguments
///
/// * `headers` - Map of HTTP headers (case-insensitive keys recommended)
/// * `fallback_ip` - Optional fallback IP address (usually the remote socket address)
///
/// # Examples
///
/// ```rust
/// use real::{extract_real_ip, HeaderMap};
/// use std::collections::HashMap;
///
/// let mut headers = HashMap::new();
/// headers.insert("x-real-ip".to_string(), "192.168.1.100".to_string());
///
/// let ip = extract_real_ip(&headers, Some("127.0.0.1".to_string()));
/// assert_eq!(ip, Some("192.168.1.100".parse().unwrap()));
/// ```
pub fn extract_real_ip(headers: &HeaderMap, fallback_ip: Option<String>) -> Option<IpAddr> {
    let extractor = IpExtractor::default().trust_private_ips(true);
    extractor.extract(headers, fallback_ip)
}

/// Extract real IP with strict validation (no private IPs from headers).
pub fn extract_real_ip_strict(headers: &HeaderMap, fallback_ip: Option<String>) -> Option<IpAddr> {
    let extractor = IpExtractor::default().trust_private_ips(false);
    extractor.extract(headers, fallback_ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_x_real_ip() {
        let mut headers = HashMap::new();
        headers.insert("x-real-ip".to_string(), "192.168.1.100".to_string());

        let ip = extract_real_ip(&headers, None);
        assert_eq!(ip, Some("192.168.1.100".parse().unwrap()));
    }

    #[test]
    fn test_extract_x_forwarded_for() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-forwarded-for".to_string(),
            "203.0.113.1, 192.168.1.1".to_string(),
        );

        let ip = extract_real_ip(&headers, None);
        assert_eq!(ip, Some("203.0.113.1".parse().unwrap()));
    }

    #[test]
    fn test_fallback_ip() {
        let headers = HashMap::new();
        let ip = extract_real_ip(&headers, Some("127.0.0.1".to_string()));
        assert_eq!(ip, Some("127.0.0.1".parse().unwrap()));
    }

    #[test]
    fn test_no_ip_found() {
        let headers = HashMap::new();
        let ip = extract_real_ip(&headers, None);
        assert_eq!(ip, None);
    }

    #[test]
    fn test_strict_mode_rejects_private() {
        let mut headers = HashMap::new();
        headers.insert("x-real-ip".to_string(), "192.168.1.100".to_string());

        let ip = extract_real_ip_strict(&headers, Some("203.0.113.1".to_string()));
        assert_eq!(ip, Some("203.0.113.1".parse().unwrap()));
    }
}
