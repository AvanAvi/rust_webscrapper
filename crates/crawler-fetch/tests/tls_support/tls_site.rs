//! Serves bounded HTTP/1 responses over real TLS on loopback, alongside the plaintext mock-site.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use mock_site::MockSite;
use rustls::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use url::Url;

use super::certificates::BoxError;

pub struct TlsSite {
    pub o_plain: MockSite,
    pub o_requests: Arc<AtomicUsize>,
    u_port: u16,
    jh_server: JoinHandle<()>,
}

impl TlsSite {
    pub async fn start(o_config: ServerConfig) -> Result<Self, BoxError> {
        let o_plain = MockSite::start().await?;
        let o_listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
        let u_port = o_listener.local_addr()?.port();
        let o_requests = Arc::new(AtomicUsize::new(0));
        let o_server = TlsServer {
            o_acceptor: TlsAcceptor::from(Arc::new(o_config)),
            o_requests: Arc::clone(&o_requests),
            u_port,
            u_plain_port: o_plain.port(),
        };
        let jh_server = tokio::spawn(o_server.serve(o_listener));
        Ok(Self {
            o_plain,
            o_requests,
            u_port,
            jh_server,
        })
    }

    pub fn url(&self, str_host: &str, str_path: &str) -> Result<Url, BoxError> {
        Ok(Url::parse(&format!(
            "https://{str_host}:{}{str_path}",
            self.u_port
        ))?)
    }
}

impl Drop for TlsSite {
    fn drop(&mut self) {
        self.jh_server.abort();
    }
}

struct TlsServer {
    o_acceptor: TlsAcceptor,
    o_requests: Arc<AtomicUsize>,
    u_port: u16,
    u_plain_port: u16,
}

impl TlsServer {
    async fn serve(self, o_listener: TcpListener) {
        while let Ok((o_stream, _)) = o_listener.accept().await {
            let _ = tokio::time::timeout(Duration::from_secs(3), self.answer(o_stream)).await;
        }
    }

    async fn answer(&self, o_stream: TcpStream) -> Result<(), BoxError> {
        let mut o_tls = self.o_acceptor.accept(o_stream).await?;
        let str_request = self.read_headers(&mut o_tls).await?;
        self.o_requests.fetch_add(1, Ordering::SeqCst);
        let str_target = str_request
            .split_whitespace()
            .nth(1)
            .ok_or("request target")?;
        let str_path = str_target.split('?').next().ok_or("request path")?;
        let str_response = self.response(str_path);
        o_tls.write_all(str_response.as_bytes()).await?;
        o_tls.shutdown().await?;
        Ok(())
    }

    async fn read_headers(&self, o_tls: &mut TlsStream<TcpStream>) -> Result<String, BoxError> {
        let mut by_headers = Vec::new();
        while by_headers.len() < 8192 {
            let u_byte = o_tls.read_u8().await?;
            by_headers.push(u_byte);
            if by_headers.ends_with(b"\r\n\r\n") {
                return Ok(String::from_utf8(by_headers)?);
            }
        }
        Err("test request headers exceed cap".into())
    }

    fn response(&self, str_path: &str) -> String {
        let opt_str_location = match str_path {
            "/down" | "/robots.txt" => Some(format!(
                "http://tls.mock:{}/landed?private_query=secret_query",
                self.u_plain_port
            )),
            "/up" => Some(format!("https://tls.mock:{}/landed", self.u_port)),
            _ => None,
        };
        if let Some(str_location) = opt_str_location {
            return format!(
                "HTTP/1.1 302 Found\r\nLocation: {str_location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
        }
        let str_status = if str_path == "/landed" {
            "200 OK"
        } else {
            "404 Not Found"
        };
        format!("HTTP/1.1 {str_status}\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
    }
}
