/* src/middleware.rs */

use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{Request, Response, request::Parts},
};
use futures_util::future::BoxFuture;
use std::{
    net::{IpAddr, SocketAddr},
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::extractor::{HeaderMap as InnerHeaderMap, IpExtractor};

/// Extension that holds the extracted real IP address.
#[derive(Debug, Clone)]
pub struct RealIp(pub IpAddr);

impl RealIp {
    /// Get the IP address.
    pub fn ip(&self) -> IpAddr {
        self.0
    }
}

/// A layer that extracts the real IP address from a request and inserts it into
/// the request extensions, making it available for subsequent handlers and extractors.
#[derive(Debug, Clone)]
pub struct RealIpLayer {
    extractor: IpExtractor,
}

impl Default for RealIpLayer {
    fn default() -> Self {
        Self {
            // Default behavior: trust private IPs from headers.
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

/// The `tower::Service` that implements the real IP extraction logic.
#[derive(Debug, Clone)]
pub struct RealIpService<S> {
    inner: S,
    extractor: IpExtractor,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RealIpService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let extractor = self.extractor.clone();

        Box::pin(async move {
            let fallback_ip = req
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|connect_info| connect_info.0.ip().to_string());

            let header_map = headers_to_map(req.headers());

            if let Some(real_ip) = extractor.extract(&header_map, fallback_ip) {
                req.extensions_mut().insert(RealIp(real_ip));
            }

            inner.call(req).await
        })
    }
}

/// Convert Axum headers to our internal header map format.
fn headers_to_map(headers: &axum::http::HeaderMap) -> InnerHeaderMap {
    let mut map = InnerHeaderMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            map.insert(name.as_str().to_lowercase(), value_str.to_string());
        }
    }
    map
}

/// Axum extractor for the real IP address.
impl<S> FromRequestParts<S> for RealIp
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let _ = state;

        if let Some(real_ip) = parts.extensions.get::<RealIp>() {
            return Ok(real_ip.clone());
        }

        let fallback_ip = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect_info| connect_info.0.ip().to_string());

        let header_map = headers_to_map(&parts.headers);

        let extractor = IpExtractor::default().trust_private_ips(false);
        if let Some(real_ip) = extractor.extract(&header_map, fallback_ip) {
            return Ok(RealIp(real_ip));
        }

        Ok(RealIp(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))))
    }
}
