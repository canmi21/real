# Real IP Extractor

A lightweight Rust library for extracting the real client IP address from HTTP requests, supporting common forwarding headers such as `X-Real-IP` and `X-Forwarded-For`, with a fallback to the remote socket address.

## Features

- Extract real IP from various HTTP headers (`X-Real-IP`, `X-Forwarded-For`, `CF-Connecting-IP`, etc.)
- Configurable header priority and IP validation rules
- Optional trust for private IPs from headers
- Support for `X-Forwarded-For` chain parsing (first or last IP)
- Fallback to remote socket address
- Optional Axum middleware and extractor integration (via the `axum` feature)
- Lightweight and dependency-minimal
- Comprehensive test suite

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
real = "0.1"
```

For Axum integration, enable the `axum` feature:

```toml
[dependencies]
real = { version = "0.1", features = ["axum"] }
```

## Usage

### Basic Usage

Extract the real IP address from HTTP headers with a fallback to the remote socket address:

```rust
use real::{extract_real_ip, HeaderMap};
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("x-real-ip".to_string(), "203.0.113.45".to_string());

let ip = extract_real_ip(&headers, Some("127.0.0.1".to_string()));
assert_eq!(ip, Some("203.0.113.45".parse().unwrap()));
```

### Custom Configuration

Create a custom `IpExtractor` with specific settings:

```rust
use real::{IpExtractor, HeaderMap};
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("custom-real-ip".to_string(), "203.0.113.200".to_string());

let extractor = IpExtractor::new()
    .with_headers(vec!["custom-real-ip".to_string()])
    .trust_private_ips(false)
    .use_first_forwarded(true);

let ip = extractor.extract(&headers, None);
assert_eq!(ip, Some("203.0.113.200".parse().unwrap()));
```

### Axum Integration

Use the `RealIpLayer` middleware to automatically extract the real IP and make it available in your handlers:

```rust
use axum::{Router, routing::get, extract::ConnectInfo};
use real::{RealIp, RealIpLayer};
use std::net::SocketAddr;

async fn handler(real_ip: RealIp) -> String {
    format!("Your real IP is: {}", real_ip.ip())
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        .layer(RealIpLayer::default());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
```

Run the Axum example with:

```bash
cargo run --example axum
```

Then test with curl:

```bash
curl -H "X-Real-IP: 203.0.113.42" http://localhost:3000
```

### Strict Mode

Use `extract_real_ip_strict` to reject private IPs from headers:

```rust
use real::{extract_real_ip_strict, HeaderMap};
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("x-real-ip".to_string(), "192.168.1.100".to_string());

let ip = extract_real_ip_strict(&headers, Some("203.0.113.50".to_string()));
assert_eq!(ip, Some("203.0.113.50".parse().unwrap()));
```

### Examples

The repository includes two examples:

1. **`demo.rs`**: Demonstrates various IP extraction scenarios using the library's core functionality.
   Run with:
   ```bash
   cargo run --example demo
   ```

2. **`axum.rs`**: Shows how to use the library with Axum middleware and extractors.
   Run with:
   ```bash
   cargo run --example axum --features="axum"
   ```

## Configuration Options

The `IpExtractor` struct allows customization of:

- **Header Priority**: Specify which headers to check and in what order.
- **Private IP Trust**: Control whether private IPs (e.g., `192.168.x.x`) from headers are trusted.
- **X-Forwarded-For Behavior**: Choose whether to use the first or last IP in the `X-Forwarded-For` chain.

## Error Handling

The library defines a `RealIpError` enum for handling errors:

```rust
use real::{Result, RealIpError};

match extract_real_ip(&headers, None) {
    Ok(Some(ip)) => println!("Extracted IP: {}", ip),
    Ok(None) => println!("No valid IP found"),
    Err(RealIpError::InvalidIpFormat(err)) => println!("Invalid IP format: {}", err),
    Err(RealIpError::NoValidIp) => println!("No valid IP address found"),
}
```

## Testing

Run the test suite with:

```bash
cargo test
```

The library includes comprehensive tests for IP extraction, header parsing, and Axum integration.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please submit issues or pull requests to the [GitHub repository](https://github.com/canmi21/real).