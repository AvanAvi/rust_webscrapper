//! Builds the bodies served by the scenarios.

/// Size of the gzip bomb before compression (zeros compress about 1000:1).
pub const U_BOMB_BYTES: usize = 64 * 1024 * 1024;
/// Size of the large compressible text (compresses only a few times, so the cap trips first).
pub const U_BIG_TEXT_BYTES: usize = 8 * 1024 * 1024;
/// Number of links on the link-explosion page.
const U_LINK_COUNT: usize = 20_000;
/// Word list for the large text.
const VEC_STR_WORDS: [&str; 16] = [
    "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india", "juliet",
    "kilo", "lima", "mike", "november", "oscar", "papa",
];

/// Factory for scenario bodies.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScenarioBodies;

impl ScenarioBodies {
    /// Creates the factory.
    pub fn new() -> Self {
        Self
    }

    /// Returns a page with many links.
    pub fn link_explosion(&self) -> String {
        let str_links: String = (0..U_LINK_COUNT)
            .map(|u_i| format!("<a href=\"/p/{u_i}\">{u_i}</a>\n"))
            .collect();
        format!("<html><body>{str_links}</body></html>")
    }

    /// Returns broken HTML with invalid UTF-8.
    pub fn malformed(&self) -> Vec<u8> {
        let mut by_page =
            b"<html><head><title>broken</ti<body><div><p>unclosed <a href='x>".to_vec();
        by_page.extend_from_slice(&[0xff, 0xfe, 0xc3, 0x28]);
        by_page.extend_from_slice(b"<meta name=robots content=noai<script>if(a<b)</div></html");
        by_page
    }

    /// Returns deterministic word text of the given size.
    pub fn word_text(&self, u_bytes: usize) -> Vec<u8> {
        let mut by_text = Vec::with_capacity(u_bytes + 16);
        let mut u_state: u64 = 0x2545_f491_4f6c_dd1d;
        while by_text.len() < u_bytes {
            u_state = u_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let str_word = VEC_STR_WORDS[(u_state >> 60) as usize];
            by_text.extend_from_slice(str_word.as_bytes());
            by_text.push(b' ');
        }
        by_text.truncate(u_bytes);
        by_text
    }
}
