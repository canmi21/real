/* src/error.rs */

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
}
