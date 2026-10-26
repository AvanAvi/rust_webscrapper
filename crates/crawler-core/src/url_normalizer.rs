//! Turns URL text into one canonical form used as the deduplication key.

use url::Url;

use crate::core_error::CoreError;
use crate::percent_normalizer::PercentNormalizer;

/// Schemes the crawler accepts.
const VEC_STR_ALLOWED_SCHEMES: [&str; 2] = ["http", "https"];

/// Normalizes http and https URLs so equivalent spellings collapse to one key.
///
/// Parsing (WHATWG rules, via the `url` crate) lowercases the scheme and host, converts
/// IDNA hosts to ASCII, drops default ports, and resolves dot segments. This type then
/// drops the fragment and normalizes percent-encoding in the path and query. Trailing
/// slashes and query parameter order are kept, because servers may treat them as distinct.
#[derive(Debug, Clone, Copy, Default)]
pub struct UrlNormalizer {
    /// Percent-encoding normalizer for path and query.
    o_percent: PercentNormalizer,
}

impl UrlNormalizer {
    /// Creates a normalizer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the normalized URL, or an error for invalid, non-http(s), or credentialed URLs.
    pub fn normalize(&self, str_url: &str) -> Result<Url, CoreError> {
        let mut url_parsed = Url::parse(str_url).map_err(|err_parse| CoreError::InvalidUrl {
            str_reason: err_parse.to_string(),
        })?;
        self.check_allowed(&url_parsed)?;
        url_parsed.set_fragment(None);
        let str_path = self.o_percent.normalize(url_parsed.path());
        url_parsed.set_path(&str_path);
        let opt_str_query = url_parsed
            .query()
            .map(|str_query| self.o_percent.normalize(str_query));
        url_parsed.set_query(opt_str_query.as_deref());
        Ok(url_parsed)
    }

    /// Returns the normalized URL as the string key used for deduplication.
    pub fn normalize_key(&self, str_url: &str) -> Result<String, CoreError> {
        Ok(self.normalize(str_url)?.into())
    }

    /// Rejects schemes other than http(s), URLs without a host, and embedded credentials.
    fn check_allowed(&self, url_parsed: &Url) -> Result<(), CoreError> {
        let str_scheme = url_parsed.scheme();
        if !VEC_STR_ALLOWED_SCHEMES.contains(&str_scheme) {
            return Err(CoreError::UnsupportedScheme {
                str_scheme: str_scheme.to_owned(),
            });
        }
        if url_parsed.host_str().is_none_or(str::is_empty) {
            return Err(CoreError::InvalidUrl {
                str_reason: String::from("missing host"),
            });
        }
        if !url_parsed.username().is_empty() || url_parsed.password().is_some() {
            let str_redacted_url = self.redact(url_parsed);
            return Err(CoreError::CredentialsInUrl { str_redacted_url });
        }
        Ok(())
    }

    /// Returns the URL without user info, query, or fragment, safe to show in errors and logs.
    fn redact(&self, url_parsed: &Url) -> String {
        let mut url_redacted = url_parsed.clone();
        let b_cleared =
            url_redacted.set_username("").is_ok() && url_redacted.set_password(None).is_ok();
        if !b_cleared {
            return String::from("<redacted>");
        }
        url_redacted.set_query(None);
        url_redacted.set_fragment(None);
        url_redacted.into()
    }
}
