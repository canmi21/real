/* examples/axum.rs */

use axum::{
    Router,
    extract::ConnectInfo,
    http::StatusCode,
    response::{Html, Json},
    routing::get,
};
use real::{IpExtractor, RealIp, RealIpLayer};
use serde_json::json;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = create_app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server starting on http://localhost:3000");
    println!("Test endpoints:");
    println!("  • GET /              - Hello World with IP info (using default layer)");
    println!("  • GET /ip            - JSON response with IP details (using default layer)");
    println!("  • GET /strict        - JSON response with strict IP validation");
    println!("  • GET /debug         - Debug endpoint showing all connection info");
    println!();
    println!("Test with headers:");
    println!("  curl -H 'X-Real-IP: 203.0.113.42' http://localhost:3000/ip");
    println!("  curl -H 'X-Forwarded-For: 198.51.100.1, 192.168.1.1' http://localhost:3000/strict");
    println!("  curl -H 'CF-Connecting-IP: 192.0.2.100' http://localhost:3000/ip");
    println!();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

fn create_app() -> Router {
    // keep your ServiceBuilder layer if you plan to add middleware later
    let app = Router::new().layer(ServiceBuilder::new());

    let default_layer = RealIpLayer::default();
    let default_router = Router::new()
        .route("/", get(hello_handler))
        .route("/ip", get(ip_handler))
        .layer(default_layer);

    let strict_layer = RealIpLayer::strict();
    let strict_router = Router::new()
        .route("/", get(ip_strict_handler))
        .layer(strict_layer);

    // merge the default (root) router instead of nesting it at "/"
    app.merge(default_router)
        .nest("/strict", strict_router)
        .route("/debug", get(debug_handler))
}

/// Basic hello world handler that shows the extracted real IP
async fn hello_handler(real_ip: RealIp) -> Html<String> {
    let ip = real_ip.ip();
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Real IP Demo</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }}
                .container {{ max-width: 600px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                .ip {{ color: #2563eb; font-weight: bold; font-size: 1.2em; }}
                .code {{ background: #f3f4f6; padding: 10px; border-radius: 5px; margin: 10px 0; font-family: monospace; }}
                .endpoint {{ margin: 20px 0; }}
                .endpoint h3 {{ color: #374151; margin-bottom: 10px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Hello, World!</h1>
                <p>Your real IP address is: <span class="ip">{}</span></p>
                <h2>Try these endpoints:</h2>
                <div class="endpoint">
                    <h3>JSON IP Info</h3>
                    <div class="code">GET /ip</div>
                    <p>Returns detailed IP information in JSON format</p>
                </div>
                <div class="endpoint">
                    <h3>Strict IP Validation</h3>
                    <div class="code">GET /strict</div>
                    <p>Uses strict validation that rejects private IPs from headers</p>
                </div>
                <div class="endpoint">
                    <h3>Debug Information</h3>
                    <div class="code">GET /debug</div>
                    <p>Shows detailed connection and header information</p>
                </div>
                <h2>Test with custom headers:</h2>
                <div class="code">curl -H 'X-Real-IP: 203.0.113.42' http://localhost:3000/ip</div>
                <div class="code">curl -H 'X-Forwarded-For: 198.51.100.1, 192.168.1.1' http://localhost:3000/strict</div>
                <div class="code">curl -H 'CF-Connecting-IP: 192.0.2.100' http://localhost:3000/ip</div>
            </div>
        </body>
        </html>
        "#,
        ip
    );

    Html(html)
}

/// Handler that returns IP information in JSON format
async fn ip_handler(real_ip: RealIp) -> Json<serde_json::Value> {
    let ip = real_ip.ip();

    Json(json!({
        "real_ip": ip.to_string(),
        "ip_version": match ip {
            std::net::IpAddr::V4(_) => "IPv4",
            std::net::IpAddr::V6(_) => "IPv6",
        },
        "is_loopback": ip.is_loopback(),
        "is_private": match ip {
            std::net::IpAddr::V4(ipv4) => ipv4.is_private(),
            std::net::IpAddr::V6(_) => false, // Simplified for demo
        },
        "middleware": "default",
        "trusts_private_ips": true
    }))
}

/// Handler with strict IP validation
async fn ip_strict_handler(real_ip: RealIp) -> Json<serde_json::Value> {
    let ip = real_ip.ip();

    Json(json!({
        "real_ip": ip.to_string(),
        "ip_version": match ip {
            std::net::IpAddr::V4(_) => "IPv4",
            std::net::IpAddr::V6(_) => "IPv6",
        },
        "is_loopback": ip.is_loopback(),
        "is_private": match ip {
            std::net::IpAddr::V4(ipv4) => ipv4.is_private(),
            std::net::IpAddr::V6(_) => false,
        },
        "middleware": "strict",
        "trusts_private_ips": false,
        "note": "This endpoint rejects private IPs from headers and falls back to connection IP"
    }))
}

/// Debug handler showing detailed connection information
async fn debug_handler(
    real_ip: RealIp,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let real_ip_addr = real_ip.ip();

    // Extract relevant headers for IP detection
    let mut ip_headers = std::collections::HashMap::new();

    let ip_header_names = [
        "x-real-ip",
        "x-forwarded-for",
        "x-forwarded",
        "cf-connecting-ip",
        "forwarded-for",
        "forwarded",
    ];

    for header_name in &ip_header_names {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(value_str) = header_value.to_str() {
                ip_headers.insert(*header_name, value_str.to_string());
            }
        }
    }

    Ok(Json(json!({
        "extracted_real_ip": real_ip_addr.to_string(),
        "connection_info": {
            "remote_addr": addr.to_string(),
            "remote_ip": addr.ip().to_string(),
            "remote_port": addr.port(),
        },
        "ip_related_headers": ip_headers,
        "all_headers": headers.iter()
            .filter_map(|(name, value)| {
                value.to_str().ok().map(|v| (name.to_string(), v.to_string()))
            })
            .collect::<std::collections::HashMap<String, String>>(),
        "analysis": {
            "ip_source": if ip_headers.is_empty() {
                "connection_fallback"
            } else {
                "header_extraction"
            },
            "header_count": ip_headers.len(),
            "ip_matches_connection": real_ip_addr == addr.ip(),
        }
    })))
}
