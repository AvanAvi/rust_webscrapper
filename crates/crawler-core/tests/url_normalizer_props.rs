//! Property tests: normalization is idempotent, collapses equivalent forms, and never merges distinct URLs.

use crawler_core::UrlNormalizer;
use proptest::prelude::*;

/// Parts of a generated URL in its plain spelling.
#[derive(Debug, Clone)]
struct UrlParts {
    /// `http` or `https`.
    str_scheme: String,
    /// Lowercase ASCII host.
    str_host: String,
    /// Path segments of unreserved characters, never `.` or `..`.
    vec_str_segments: Vec<String>,
    /// Query pairs in order.
    vec_o_query: Vec<(String, String)>,
}

impl UrlParts {
    /// Returns the URL text, spelling each path character through the closure.
    fn render<F: Fn(char, usize) -> String>(
        &self,
        str_scheme: &str,
        str_host: &str,
        cb_char: F,
    ) -> String {
        let str_path: String = self
            .vec_str_segments
            .iter()
            .map(|str_seg| self.render_segment(str_seg, &cb_char))
            .collect();
        format!("{str_scheme}://{str_host}{str_path}/{}", self.query())
    }

    /// Returns one `/segment` with each character spelled through the closure.
    fn render_segment<F: Fn(char, usize) -> String>(&self, str_seg: &str, cb_char: &F) -> String {
        let str_enc: String = str_seg
            .chars()
            .enumerate()
            .map(|(u_i, ch_c)| cb_char(ch_c, u_i))
            .collect();
        format!("/{str_enc}")
    }

    /// Returns the query part including `?`, or an empty string.
    fn query(&self) -> String {
        if self.vec_o_query.is_empty() {
            return String::new();
        }
        let vec_str_pairs: Vec<String> = self
            .vec_o_query
            .iter()
            .map(|(str_k, str_v)| format!("{str_k}={str_v}"))
            .collect();
        format!("?{}", vec_str_pairs.join("&"))
    }

    /// Returns the plain spelling.
    fn plain(&self) -> String {
        self.render(&self.str_scheme, &self.str_host, |ch_c, _| ch_c.to_string())
    }

    /// Returns `scheme://host` without a path.
    fn origin(&self) -> String {
        format!("{}://{}", self.str_scheme, self.str_host)
    }
}

/// Generates URL parts.
fn url_parts() -> impl Strategy<Value = UrlParts> {
    (
        prop_oneof![Just("http"), Just("https")],
        "[a-z][a-z0-9]{0,6}(\\.[a-z][a-z0-9]{0,6}){0,2}\\.(com|org|net)",
        prop::collection::vec("[A-Za-z0-9_~-][A-Za-z0-9._~-]{0,6}", 0..4),
        prop::collection::vec(("[a-z]{1,4}", "[a-z0-9]{0,4}"), 0..3),
    )
        .prop_map(
            |(str_scheme, str_host, vec_str_segments, vec_o_query)| UrlParts {
                str_scheme: str_scheme.to_owned(),
                str_host,
                vec_str_segments,
                vec_o_query,
            },
        )
}

/// Returns the key for a URL; a rejection fails the case (inputs here are always valid).
fn key(str_url: &str) -> Result<String, TestCaseError> {
    UrlNormalizer::new()
        .normalize_key(str_url)
        .map_err(|err_url| TestCaseError::fail(format!("{str_url}: {err_url}")))
}

/// Returns a percent-escape for an ASCII character in the chosen hex case.
fn escape(ch_c: char, b_upper: bool) -> String {
    let str_hex = format!("%{:02x}", u32::from(ch_c));
    if b_upper {
        str_hex.to_uppercase()
    } else {
        str_hex
    }
}

/// Returns the plain URL respelled with upper-case scheme and host, an optional explicit
/// default port, escapes on masked characters, and an optional fragment.
fn respell(
    o_parts: &UrlParts,
    vec_b_mask: &[bool],
    b_upper: bool,
    b_extras: (bool, bool),
) -> String {
    let (b_port, b_frag) = b_extras;
    let str_default_port = if o_parts.str_scheme == "http" {
        ":80"
    } else {
        ":443"
    };
    let str_port = if b_port { str_default_port } else { "" };
    let str_host = format!("{}{str_port}", o_parts.str_host.to_uppercase());
    let cb_char = |ch_c: char, u_i: usize| {
        if vec_b_mask[u_i % vec_b_mask.len()] {
            escape(ch_c, b_upper)
        } else {
            ch_c.to_string()
        }
    };
    let str_url = o_parts.render(&o_parts.str_scheme.to_uppercase(), &str_host, cb_char);
    if b_frag {
        format!("{str_url}#Frag%20ment")
    } else {
        str_url
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn test_equivalent_spellings_collapse(
        o_parts in url_parts(),
        vec_b_mask in prop::collection::vec(any::<bool>(), 1..16),
        b_upper in any::<bool>(),
        b_extras in any::<(bool, bool)>(),
    ) {
        let str_variant = respell(&o_parts, &vec_b_mask, b_upper, b_extras);
        prop_assert_eq!(key(&str_variant)?, key(&o_parts.plain())?);
    }

    #[test]
    fn test_normalization_is_idempotent(o_parts in url_parts(), str_tail in "[ -~]{0,30}") {
        for str_url in [o_parts.plain(), format!("http://example.com/{str_tail}")] {
            if let Ok(str_once) = UrlNormalizer::new().normalize_key(&str_url) {
                prop_assert_eq!(key(&str_once)?, str_once);
            }
        }
    }

    #[test]
    fn test_encoded_slash_never_merges_with_slash(
        o_parts in url_parts(),
        str_left in "[a-z]{1,5}",
        str_right in "[a-z]{1,5}",
    ) {
        let str_base = o_parts.origin();
        let str_slash = key(&format!("{str_base}/{str_left}/{str_right}"))?;
        prop_assert_ne!(key(&format!("{str_base}/{str_left}%2F{str_right}"))?, str_slash.clone());
        prop_assert_ne!(key(&format!("{str_base}/{str_left}%2f{str_right}"))?, str_slash);
    }

    #[test]
    fn test_trailing_slash_and_path_case_stay_distinct(
        o_parts in url_parts(),
        str_seg in "[a-z][a-zA-Z0-9]{0,6}",
    ) {
        let str_base = o_parts.origin();
        let str_plain = key(&format!("{str_base}/{str_seg}"))?;
        prop_assert_ne!(key(&format!("{str_base}/{str_seg}/"))?, str_plain.clone());
        prop_assert_ne!(key(&format!("{str_base}/{}", str_seg.to_uppercase()))?, str_plain);
    }

    #[test]
    fn test_query_order_stays_distinct(
        o_parts in url_parts(),
        str_a in "[a-m]{1,4}",
        str_b in "[n-z]{1,4}",
    ) {
        let str_base = format!("{}/p", o_parts.origin());
        let str_ab = key(&format!("{str_base}?{str_a}=1&{str_b}=2"))?;
        prop_assert_ne!(key(&format!("{str_base}?{str_b}=2&{str_a}=1"))?, str_ab);
    }

    #[test]
    fn test_invalid_escapes_are_kept_verbatim(o_parts in url_parts(), str_bad in "%[g-z][g-z]") {
        let str_key = key(&format!("{}/x{str_bad}?q={str_bad}", o_parts.origin()))?;
        prop_assert!(str_key.ends_with(&format!("/x{str_bad}?q={str_bad}")), "{}", str_key);
    }
}
