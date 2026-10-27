//! Starts the mock site on 127.0.0.1 and exposes its request log.

use std::net::{Ipv4Addr, SocketAddr};

use axum::Router;
use axum::extract::Request;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower_http::compression::CompressionLayer;

use crate::raw_server::RawServer;
use crate::recorded_request::RecordedRequest;
use crate::request_log::RequestLog;
use crate::scenario_router::ScenarioRouter;

/// A running mock site: an HTTP server with virtual-host scenarios (gzip when asked) and a raw
/// TCP server for byte-level misbehaviour, both on 127.0.0.1. Dropping it stops both.
#[derive(Debug)]
pub struct MockSite {
    /// Port of the HTTP server.
    u_port: u16,
    /// Port of the raw server.
    u_raw_port: u16,
    /// Requests received by both servers.
    o_log: RequestLog,
    /// HTTP server task.
    jh_http: JoinHandle<()>,
    /// Raw server task.
    jh_raw: JoinHandle<()>,
}

impl MockSite {
    /// Binds both servers to ephemeral loopback ports and starts them.
    pub async fn start() -> std::io::Result<Self> {
        let o_http_listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
        let o_raw_listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
        let u_port = o_http_listener.local_addr()?.port();
        let u_raw_port = o_raw_listener.local_addr()?.port();
        let o_log = RequestLog::new();
        let o_app = Self::app(ScenarioRouter::new(u_port, o_log.clone()));
        let jh_http = tokio::spawn(async move {
            let _ = axum::serve(o_http_listener, o_app).await;
        });
        let jh_raw = tokio::spawn(RawServer::new(o_log.clone()).serve(o_raw_listener));
        Ok(Self {
            u_port,
            u_raw_port,
            o_log,
            jh_http,
            jh_raw,
        })
    }

    /// Returns the axum app: every request goes to the scenario router, gzip when accepted.
    fn app(o_router: ScenarioRouter) -> Router {
        Router::new()
            .fallback(move |o_request: Request| {
                let o_router = o_router.clone();
                async move { o_router.respond(&o_request) }
            })
            .layer(CompressionLayer::new())
    }

    /// Returns the HTTP server port.
    pub fn port(&self) -> u16 {
        self.u_port
    }

    /// Returns a URL on the HTTP server for a virtual host and path.
    pub fn url(&self, str_host: &str, str_path: &str) -> String {
        format!("http://{str_host}:{}{str_path}", self.u_port)
    }

    /// Returns a URL on the raw server for a virtual host and path.
    pub fn raw_url(&self, str_host: &str, str_path: &str) -> String {
        format!("http://{str_host}:{}{str_path}", self.u_raw_port)
    }

    /// Returns every request received so far.
    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.o_log.all()
    }

    /// Returns how many requests had this path.
    pub fn count_path(&self, str_path: &str) -> usize {
        self.o_log
            .all()
            .iter()
            .filter(|o_r| o_r.str_path == str_path)
            .count()
    }
}

impl Drop for MockSite {
    /// Stops both servers.
    fn drop(&mut self) {
        self.jh_http.abort();
        self.jh_raw.abort();
    }
}
