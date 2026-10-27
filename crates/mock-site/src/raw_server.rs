//! Serves misbehaving responses that a normal HTTP server cannot: slow drip, lying length, endless body.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::request_log::RequestLog;

/// Largest request head read, in bytes.
const U_MAX_HEAD_BYTES: usize = 8192;
/// Pause between dripped bytes.
const DUR_DRIP_INTERVAL: Duration = Duration::from_millis(300);

/// Raw TCP server for scenarios that need byte-level control.
#[derive(Debug, Clone)]
pub struct RawServer {
    /// Request log shared with the HTTP server.
    o_log: RequestLog,
    /// TCP connections accepted, counted before anything is read.
    o_accepted: Arc<AtomicUsize>,
}

impl RawServer {
    /// Creates a raw server.
    pub fn new(o_log: RequestLog, o_accepted: Arc<AtomicUsize>) -> Self {
        Self { o_log, o_accepted }
    }

    /// Accepts connections until the listener fails.
    pub async fn serve(self, o_listener: TcpListener) {
        while let Ok((o_stream, _)) = o_listener.accept().await {
            self.o_accepted.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(self.clone().handle(o_stream));
        }
    }

    /// Reads the request head, logs it, and plays the scenario for its Host.
    async fn handle(self, mut o_stream: TcpStream) {
        let str_head = self.read_head(&mut o_stream).await;
        let str_host = self.host_of(&str_head);
        let str_path = str_head.split_whitespace().nth(1).unwrap_or("/").to_owned();
        self.o_log.record(&str_host, &str_path);
        let (str_status, by_chunk, dur_pause) = self.script_for(&str_host);
        if o_stream.write_all(str_status.as_bytes()).await.is_ok() {
            self.stream_forever(&mut o_stream, &by_chunk, dur_pause)
                .await;
        }
    }

    /// Returns the status and headers, the repeated chunk, and the pause for a host.
    fn script_for(&self, str_host: &str) -> (&'static str, Vec<u8>, Duration) {
        match str_host {
            "drip.mock" => (
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
                vec![b'x'],
                DUR_DRIP_INTERVAL,
            ),
            "huge.mock" => (
                "HTTP/1.1 200 OK\r\nContent-Length: 1073741824\r\n\r\n",
                vec![0u8; 65_536],
                Duration::ZERO,
            ),
            _ => (
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
                vec![b'z'; 65_536],
                Duration::ZERO,
            ),
        }
    }

    /// Writes the chunk repeatedly until the client goes away.
    async fn stream_forever(&self, o_stream: &mut TcpStream, by_chunk: &[u8], dur_pause: Duration) {
        while o_stream.write_all(by_chunk).await.is_ok() {
            if !dur_pause.is_zero() {
                tokio::time::sleep(dur_pause).await;
            }
        }
    }

    /// Reads bytes until the end of the request head or the size limit.
    async fn read_head(&self, o_stream: &mut TcpStream) -> String {
        let mut by_head = Vec::new();
        let mut by_buffer = [0u8; 1024];
        while by_head.len() < U_MAX_HEAD_BYTES && !by_head.ends_with(b"\r\n\r\n") {
            match o_stream.read(&mut by_buffer).await {
                Ok(0) | Err(_) => break,
                Ok(u_read) => {
                    by_head.extend_from_slice(by_buffer.get(..u_read).unwrap_or_default())
                }
            }
        }
        String::from_utf8_lossy(&by_head).into_owned()
    }

    /// Returns the Host header value without its port, lowercased.
    fn host_of(&self, str_head: &str) -> String {
        str_head
            .lines()
            .find_map(|str_line| {
                str_line
                    .strip_prefix("Host:")
                    .or_else(|| str_line.strip_prefix("host:"))
            })
            .map(|str_value| {
                str_value
                    .trim()
                    .split(':')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
            })
            .unwrap_or_default()
    }
}
