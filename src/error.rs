/* src/error.rs */

use std::net::AddrParseError;
use thiserror::Error;

/// Result type alias for operations that may fail with `RealIpError`.
pub type Result<T> = std::result::Result<T, RealIpError>;

/// Errors that can occur during IP extraction.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RealIpError {
    /// Invalid IP address format.
    #[error("Invalid IP address format: {0}")]
    InvalidIpFormat(String),

    /// No valid IP address found in headers or fallback.
    #[error("No valid IP address found")]
    NoValidIp,

    /// Header value contains invalid UTF-8.
    #[error("Header value contains invalid UTF-8: {0}")]
    InvalidUtf8(String),
}

impl From<AddrParseError> for RealIpError {
    fn from(err: AddrParseError) -> Self {
        RealIpError::InvalidIpFormat(err.to_string())
    }
}
