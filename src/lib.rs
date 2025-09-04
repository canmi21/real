/* src/lib.rs */
//! # Real IP Extractor
//!
//! A lightweight library for extracting the real client IP address from HTTP requests,
//! supporting common forwarding headers such as `X-Real-IP` and `X-Forwarded-For`,
//! with a fallback to the remote socket address.
//!
//! ## Features
//!
//! - Extract real IP from various HTTP headers
//! - Support for X-Real-IP, X-Forwarded-For, CF-Connecting-IP headers
//! - Fallback to remote socket address
//! - Optional Axum middleware and extractor integration via the `axum` feature
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use real::{extract_real_ip, HeaderMap};
//! use std::collections::HashMap;
//! use std::net::IpAddr;
//!
//! let mut headers = HashMap::new();
//! headers.insert("x-real-ip".to_string(), "192.168.1.100".to_string());
//!
//! let ip = extract_real_ip(&headers, Some("127.0.0.1".to_string()));
//! // The default `extract_real_ip` trusts private IPs.
//! assert_eq!(ip, Some("192.168.1.100".parse().unwrap()));
//! ```

pub mod error;
pub mod extractor;

#[cfg(feature = "axum")]
pub mod middleware;

pub use error::{RealIpError, Result};
pub use extractor::{HeaderMap, IpExtractor, extract_real_ip, extract_real_ip_strict};

#[cfg(feature = "axum")]
pub use middleware::{RealIp, RealIpLayer, RealIpService};

/// Re-export commonly used types
pub use std::net::IpAddr;
