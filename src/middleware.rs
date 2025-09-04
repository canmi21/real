/* src/middleware.rs */

use axum::{
    extract::{ConnectInfo, Request},
    http::HeaderMap,
    response::Response,
};
use futures_util::future::BoxFuture;
use std::{
    net::{IpAddr, SocketAddr},
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::extractor::IpExtractor;

/// Extension that holds the extracted real IP address.
#[derive(Debug, Clone)]
pub struct RealIp(pub IpAddr);

impl RealIp {
    /// Get the IP address.
    pub fn ip(&self) -> IpAddr {
        self.0
    }
}

/// Layer for extracting real IP addresses from HTTP requests.
///
/// This layer will examine common forwarding headers and extract the real client IP,
/// storing it as a request extension that can be accessed by handlers.
///
/// # Examples
///
/// ```rust,no_run
/// use axum::{Router, routing::get, response::Json};
/// use real::RealIpLayer;
/// use tower::ServiceBuilder;
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(ServiceBuilder::new().layer(RealIpLayer::default()));
/// ```
#[derive(Debug, Clone)]
pub struct RealIpLayer {
    extractor: IpExtractor,
}

impl Default for RealIpLayer {
    fn default() -> Self {
        Self {
            extractor: IpExtractor::default().trust_private_ips(true),
        }
    }
}

impl RealIpLayer {
    /// Create a new real IP layer with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new real IP layer with custom extractor configuration.
    pub fn with_extractor(extractor: IpExtractor) -> Self {
        Self { extractor }
    }

    /// Create a strict layer that doesn't trust private IPs from headers.
    pub fn strict() -> Self {
        Self {
            extractor: IpExtractor::default().trust_private_ips(false),
        }
    }
}

impl<S> Layer<S> for RealIpLayer {
    type Service = RealIpService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RealIpService {
            inner,
            extractor: self.extractor.clone(),
        }
    }
}

/// Service that extracts real IP addresses.
#[derive(Debug, Clone)]
pub struct RealIpService<S> {
    inner: S,
    extractor: IpExtractor,
}

impl<S> Service<Request> for RealIpService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        // Extract headers
        let headers = req.headers();
        let header_map = headers_to_map(headers);

        // Get fallback IP from connection info
        let fallback_ip = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect_info| connect_info.0.ip().to_string());

        // Extract real IP
        if let Some(real_ip) = self.extractor.extract(&header_map, fallback_ip) {
            req.extensions_mut().insert(RealIp(real_ip));
        }

        let future = self.inner.call(req);
        Box::pin(async move { future.await })
    }
}

/// Convert Axum headers to our internal header map format.
fn headers_to_map(headers: &HeaderMap) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            map.insert(name.as_str().to_lowercase(), value_str.to_string());
        }
    }

    map
}

/// Axum extractor for the real IP address.
///
/// # Examples
///
/// ```rust,no_run
/// use axum::{response::Json, routing::get, Router};
/// use real::{RealIp, RealIpLayer};
/// use serde_json::json;
///
/// async fn handler(real_ip: Option<RealIp>) -> Json<serde_json::Value> {
///     match real_ip {
///         Some(ip) => Json(json!({"ip": ip.ip().to_string()})),
///         None => Json(json!({"error": "Could not determine real IP"})),
///     }
/// }
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(RealIpLayer::default());
/// ```
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for RealIp
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(real_ip) = parts.extensions.get::<RealIp>() {
            Ok(real_ip.clone())
        } else {
            // Fallback to connection info if available
            if let Some(connect_info) = parts.extensions.get::<ConnectInfo<SocketAddr>>() {
                Ok(RealIp(connect_info.0.ip()))
            } else {
                // Default fallback
                Ok(RealIp(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))))
            }
        }
    }
}
