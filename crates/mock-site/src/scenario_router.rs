//! Answers HTTP requests by virtual host (Host header) with one scenario per host.

use axum::body::Body;
use axum::extract::Request;
use axum::http::header::{CONTENT_TYPE, HOST, LOCATION, RETRY_AFTER};
use axum::http::{HeaderName, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::request_log::RequestLog;
use crate::scenario_bodies::{ScenarioBodies, U_BIG_TEXT_BYTES, U_BOMB_BYTES};
use crate::signal_scenarios::SignalScenarios;

/// HTML content type.
const STR_HTML: &str = "text/html; charset=utf-8";

/// Scenario dispatcher; knows its own port so redirects can point back at it.
#[derive(Debug, Clone)]
pub struct ScenarioRouter {
    /// Port the HTTP listener is bound to.
    u_port: u16,
    /// Request log.
    o_log: RequestLog,
    /// Body factory.
    o_bodies: ScenarioBodies,
    /// Signal-file scenarios.
    o_signals: SignalScenarios,
}

impl ScenarioRouter {
    /// Creates a router for a port and log.
    pub fn new(u_port: u16, o_log: RequestLog) -> Self {
        Self {
            u_port,
            o_log,
            o_bodies: ScenarioBodies::new(),
            o_signals: SignalScenarios::new(u_port),
        }
    }

    /// Logs a request and answers it from its scenario.
    pub fn respond(&self, o_request: &Request) -> Response {
        let str_host_header = o_request
            .headers()
            .get(HOST)
            .and_then(|o_v| o_v.to_str().ok())
            .unwrap_or_default();
        let str_host = str_host_header
            .split(':')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let str_path = o_request.uri().path().to_owned();
        self.o_log.record(&str_host, &str_path);
        if str_path == "/landed" {
            return (StatusCode::OK, "landed").into_response();
        }
        self.dispatch(&str_host, &str_path)
    }

    /// Picks the scenario for a host.
    fn dispatch(&self, str_host: &str, str_path: &str) -> Response {
        match str_host {
            str_sig if str_sig.starts_with("sig-") => self.o_signals.respond(str_sig, str_path),
            "ok.mock" => self.ok_page(),
            "private-redirect.mock" => self.private_redirect(str_path),
            "rebind.mock" | "hops.mock" | "scope.mock" => self.redirect_chain(str_host, str_path),
            "bomb.mock" => self.html(Body::from(vec![0u8; U_BOMB_BYTES])),
            "big.mock" => self.html(Body::from(self.o_bodies.word_text(U_BIG_TEXT_BYTES))),
            "ratelimit.mock" => (
                StatusCode::TOO_MANY_REQUESTS,
                [(RETRY_AFTER, "120")],
                "slow down, please",
            )
                .into_response(),
            "malformed.mock" => self.html(Body::from(self.o_bodies.malformed())),
            "links.mock" => self.html(Body::from(self.o_bodies.link_explosion())),
            _ => (StatusCode::NOT_FOUND, "unknown mock host").into_response(),
        }
    }

    /// Returns a small page with an X-Robots-Tag header.
    fn ok_page(&self) -> Response {
        let str_page = "<html><head><title>ok</title></head><body><p>Hello from the mock site.</p></body></html>";
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, STR_HTML),
                (HeaderName::from_static("x-robots-tag"), "noai"),
            ],
            str_page,
        )
            .into_response()
    }

    /// Redirects to blocked targets: a literal 0.0.0.0, a name resolving to it, or cloud metadata.
    fn private_redirect(&self, str_path: &str) -> Response {
        let u_port = self.u_port;
        let str_location = match str_path {
            "/to-literal" => format!("http://0.0.0.0:{u_port}/landed"),
            "/to-name" => format!("http://internal.mock:{u_port}/landed"),
            "/to-metadata" => String::from("http://169.254.169.254/latest/meta-data/"),
            _ => format!("http://[::ffff:0.0.0.0]:{u_port}/landed"),
        };
        self.redirect(&str_location)
    }

    /// Redirect chains: rebinding (`/` then `/next`), endless hops, and an off-host hop.
    fn redirect_chain(&self, str_host: &str, str_path: &str) -> Response {
        match (str_host, str_path) {
            ("rebind.mock", "/") => self.redirect("/next"),
            ("scope.mock", _) => self.redirect(&format!("http://ok.mock:{}/", self.u_port)),
            ("hops.mock", _) => self.redirect(&format!("{}/x", str_path.trim_end_matches('/'))),
            _ => (
                StatusCode::OK,
                [(CONTENT_TYPE, STR_HTML)],
                "<html><body>arrived after redirect</body></html>",
            )
                .into_response(),
        }
    }

    /// Returns a 302 to a location.
    fn redirect(&self, str_location: &str) -> Response {
        (StatusCode::FOUND, [(LOCATION, str_location.to_owned())], "").into_response()
    }

    /// Returns a 200 HTML response with a body.
    fn html(&self, o_body: Body) -> Response {
        (StatusCode::OK, [(CONTENT_TYPE, STR_HTML)], o_body).into_response()
    }
}
