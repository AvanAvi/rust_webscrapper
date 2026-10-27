//! Describes who the crawler declares itself to be.

use crate::core_error::CoreError;
use crate::identity_record::IdentityRecord;
use crate::purpose::Purpose;
use crate::url_normalizer::UrlNormalizer;

/// Product tokens of other crawlers; using one would be spoofing.
const VEC_STR_RESERVED_TOKENS: [&str; 22] = [
    "googlebot",
    "google-extended",
    "googleother",
    "bingbot",
    "slurp",
    "duckduckbot",
    "baiduspider",
    "yandexbot",
    "applebot",
    "applebot-extended",
    "gptbot",
    "chatgpt-user",
    "oai-searchbot",
    "ccbot",
    "claudebot",
    "claude-web",
    "anthropic-ai",
    "perplexitybot",
    "bytespider",
    "amazonbot",
    "facebookbot",
    "meta-externalagent",
];
/// Longest product token accepted.
const U_MAX_TOKEN_BYTES: usize = 64;

/// Honest crawler identity: own product token, version, contact URL, and declared purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrawlerIdentity {
    /// RFC 9309 product token (letters, `-`, `_`), matched against robots.txt groups.
    str_product_token: String,
    /// Crawler version.
    str_version: String,
    /// Normalized contact URL.
    str_contact_url: String,
    /// Declared purpose for the whole job.
    e_purpose: Purpose,
}

impl CrawlerIdentity {
    /// Creates an identity, rejecting malformed tokens, other crawlers' tokens, and bad contacts.
    pub fn new(
        str_product_token: &str,
        str_version: &str,
        str_contact_url: &str,
        e_purpose: Purpose,
    ) -> Result<Self, CoreError> {
        Self::check_token(str_product_token)?;
        Self::check_version(str_version)?;
        let str_contact = UrlNormalizer::new()
            .normalize_key(str_contact_url)
            .map_err(|err_url| Self::invalid(&format!("contact URL: {err_url}")))?;
        Ok(Self {
            str_product_token: str_product_token.to_owned(),
            str_version: str_version.to_owned(),
            str_contact_url: str_contact,
            e_purpose,
        })
    }

    /// Returns the User-Agent header value, which names the contact URL and purpose.
    pub fn user_agent(&self) -> String {
        format!(
            "{}/{} (+{}; purpose={})",
            self.str_product_token,
            self.str_version,
            self.str_contact_url,
            self.e_purpose.key()
        )
    }

    /// Returns the product token.
    pub fn product_token(&self) -> &str {
        &self.str_product_token
    }

    /// Returns the declared purpose.
    pub fn purpose(&self) -> Purpose {
        self.e_purpose
    }

    /// Returns the receipt form of the identity.
    pub fn to_record(&self) -> IdentityRecord {
        IdentityRecord {
            str_user_agent: self.user_agent(),
            e_purpose: self.e_purpose,
            str_contact_url: self.str_contact_url.clone(),
        }
    }

    /// Rejects tokens that are not RFC 9309 identifiers or that belong to other crawlers.
    fn check_token(str_token: &str) -> Result<(), CoreError> {
        let b_identifier = !str_token.is_empty()
            && str_token.len() <= U_MAX_TOKEN_BYTES
            && str_token
                .bytes()
                .all(|u_b| u_b.is_ascii_alphabetic() || u_b == b'-' || u_b == b'_');
        if !b_identifier {
            return Err(Self::invalid(
                "product token must be 1-64 letters, '-' or '_' (RFC 9309)",
            ));
        }
        if VEC_STR_RESERVED_TOKENS
            .iter()
            .any(|str_r| str_r.eq_ignore_ascii_case(str_token))
        {
            return Err(Self::invalid("product token belongs to another crawler"));
        }
        Ok(())
    }

    /// Rejects empty or unusual version strings.
    fn check_version(str_version: &str) -> Result<(), CoreError> {
        let b_ok = !str_version.is_empty()
            && str_version.len() <= 32
            && str_version
                .bytes()
                .all(|u_b| u_b.is_ascii_alphanumeric() || matches!(u_b, b'.' | b'-' | b'+'));
        if b_ok {
            Ok(())
        } else {
            Err(Self::invalid("version must be 1-32 of [A-Za-z0-9.+-]"))
        }
    }

    /// Builds the identity error.
    fn invalid(str_reason: &str) -> CoreError {
        CoreError::InvalidIdentity {
            str_reason: str_reason.to_owned(),
        }
    }
}
