//! Serves robots.txt, ai.txt, tdmrep.json, and pages for the signal-collector scenarios (`sig-*.mock`).

use axum::http::header::{CONTENT_TYPE, LOCATION, RETRY_AFTER};
use axum::http::{HeaderName, StatusCode};
use axum::response::{AppendHeaders, IntoResponse, Response};

use crate::scenario_bodies::ScenarioBodies;

/// robots.txt of sig-ok.mock.
const STR_ROBOTS_OK: &str =
    "User-agent: *\nDisallow: /private/\nContent-Signal: search=yes, ai-train=no\n";
/// Size of the oversized robots.txt padding (more than the 1 MiB signal cap).
const U_BIG_ROBOTS_PADDING: usize = 1_500_000;
/// Exact size of the 2 MiB robots.txt (twice the signal cap).
const U_TWO_MIB_ROBOTS_BYTES: usize = 2 * 1024 * 1024;

/// Signal-file scenarios keyed by host and path; unknown combinations answer 404.
#[derive(Debug, Clone)]
pub struct SignalScenarios {
    /// Port of the HTTP server, for cross-host redirects.
    u_port: u16,
}

impl SignalScenarios {
    /// Creates the scenarios for a port.
    pub fn new(u_port: u16) -> Self {
        Self { u_port }
    }

    /// Answers one request.
    pub fn respond(&self, str_host: &str, str_path: &str) -> Response {
        match (str_host, str_path) {
            ("sig-ok.mock", "/robots.txt") => self.text(STR_ROBOTS_OK.to_owned()),
            ("sig-ok.mock", "/ai.txt") => self.text(String::from("User-Agent: *\nDisallow: *.jpg\n")),
            ("sig-ok.mock", "/.well-known/tdmrep.json") => self.json(r#"[{"location":"/reserved/*","tdm-reservation":1,"tdm-policy":"https://sig-ok.mock/policy.json"}]"#),
            ("sig-ok.mock", "/page") => self.page(),
            ("sig-500.mock", _) => (StatusCode::INTERNAL_SERVER_ERROR, "down").into_response(),
            ("sig-redirect.mock", "/robots.txt") => self.redirect(&format!("http://sig-target.mock:{}/robots.txt", self.u_port)),
            ("sig-target.mock", "/robots.txt") => self.text(String::from("User-agent: *\nDisallow: /from-target/\n")),
            ("sig-big.mock", "/robots.txt") => self.text(self.big_robots()),
            ("sig-2mib.mock", "/robots.txt") => self.text(self.sized_robots(U_TWO_MIB_ROBOTS_BYTES)),
            ("sig-ai.mock", "/ai.txt") => self.text(String::from("User-Agent: *\nDisallow: *.html\n")),
            ("sig-tdm-invalid.mock", "/.well-known/tdmrep.json") => self.json("{not json"),
            ("sig-tdm-float.mock", "/.well-known/tdmrep.json") => self.json(r#"[{"location":"/*","tdm-reservation":1.0}]"#),
            ("sig-429.mock", _) => (StatusCode::TOO_MANY_REQUESTS, [(RETRY_AFTER, "120")], "slow down").into_response(),
            ("sig-503.mock", _) => (StatusCode::SERVICE_UNAVAILABLE, [(RETRY_AFTER, "Wed, 21 Oct 2099 07:28:00 GMT")], "later").into_response(),
            _ => (StatusCode::NOT_FOUND, "not here").into_response(),
        }
    }

    /// Returns a page with two X-Robots-Tag headers and a robots meta element.
    fn page(&self) -> Response {
        let o_robots_tag = HeaderName::from_static("x-robots-tag");
        let o_headers = AppendHeaders([
            (CONTENT_TYPE, "text/html"),
            (o_robots_tag.clone(), "noindex"),
            (o_robots_tag, "noai"),
        ]);
        let str_page = "<html><head><meta name=\"robots\" content=\"noarchive\"></head><body><p>page</p></body></html>";
        (StatusCode::OK, o_headers, str_page).into_response()
    }

    /// Returns a robots.txt larger than the signal cap, with a rule only past the cap.
    fn big_robots(&self) -> String {
        let str_padding =
            String::from_utf8_lossy(&ScenarioBodies::new().word_text(U_BIG_ROBOTS_PADDING))
                .into_owned();
        format!("User-agent: *\nDisallow: /early/\n# {str_padding}\nDisallow: /late/\n")
    }

    /// Returns a robots.txt of exactly `u_total` bytes: an early rule, a long comment line,
    /// and a late rule at the very end.
    fn sized_robots(&self, u_total: usize) -> String {
        let (str_head, str_tail) = (
            "User-agent: *\nDisallow: /early/\n# ",
            "\nDisallow: /late/\n",
        );
        let mut by_body = str_head.as_bytes().to_vec();
        let by_words: Vec<u8> = ScenarioBodies::new()
            .word_text(u_total)
            .into_iter()
            .map(|u_b| if u_b == b'\n' { b' ' } else { u_b })
            .collect();
        by_body.extend(by_words);
        by_body.truncate(u_total.saturating_sub(str_tail.len()));
        by_body.extend_from_slice(str_tail.as_bytes());
        String::from_utf8_lossy(&by_body).into_owned()
    }

    /// Returns a 200 text/plain response.
    fn text(&self, str_body: String) -> Response {
        (StatusCode::OK, [(CONTENT_TYPE, "text/plain")], str_body).into_response()
    }

    /// Returns a 200 application/json response.
    fn json(&self, str_body: &'static str) -> Response {
        (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/json")],
            str_body,
        )
            .into_response()
    }

    /// Returns a 302 to a location.
    fn redirect(&self, str_location: &str) -> Response {
        (StatusCode::FOUND, [(LOCATION, str_location.to_owned())], "").into_response()
    }
}
