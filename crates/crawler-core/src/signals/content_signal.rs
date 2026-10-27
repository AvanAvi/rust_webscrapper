//! Holds the preferences stated by a Content-Signal line.

use serde::{Deserialize, Serialize};

use crate::purpose::Purpose;

/// Content-Signal preferences; `None` means the site stated no preference for that use.
///
/// Syntax per Cloudflare's Content Signals Policy (2025-09-24): one `Content-Signal:` line
/// inside a user-agent group, with comma-separated `key=yes|no` pairs for `search`,
/// `ai-input`, and `ai-train`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentSignal {
    /// Preference for building a search index.
    #[serde(rename = "search", deserialize_with = "Option::deserialize")]
    pub opt_b_search: Option<bool>,
    /// Preference for feeding content to models at inference time.
    #[serde(rename = "ai_input", deserialize_with = "Option::deserialize")]
    pub opt_b_ai_input: Option<bool>,
    /// Preference for training or fine-tuning models.
    #[serde(rename = "ai_train", deserialize_with = "Option::deserialize")]
    pub opt_b_ai_train: Option<bool>,
}

impl ContentSignal {
    /// Returns the stated preference for a purpose; archiving has no Content-Signal key.
    pub fn for_purpose(&self, e_purpose: Purpose) -> Option<bool> {
        match e_purpose {
            Purpose::Search => self.opt_b_search,
            Purpose::AiInput => self.opt_b_ai_input,
            Purpose::AiTrain => self.opt_b_ai_train,
            Purpose::Archive => None,
        }
    }

    /// Returns the Content-Signal key for a purpose, if it has one.
    pub fn key_for(e_purpose: Purpose) -> Option<&'static str> {
        match e_purpose {
            Purpose::Search => Some("search"),
            Purpose::AiInput => Some("ai-input"),
            Purpose::AiTrain => Some("ai-train"),
            Purpose::Archive => None,
        }
    }

    /// Returns both preferences combined; a `no` from either side wins.
    pub fn merge(&self, o_other: &Self) -> Self {
        Self {
            opt_b_search: Self::combine(self.opt_b_search, o_other.opt_b_search),
            opt_b_ai_input: Self::combine(self.opt_b_ai_input, o_other.opt_b_ai_input),
            opt_b_ai_train: Self::combine(self.opt_b_ai_train, o_other.opt_b_ai_train),
        }
    }

    /// Combines two stated preferences, preferring the restrictive one.
    fn combine(opt_b_left: Option<bool>, opt_b_right: Option<bool>) -> Option<bool> {
        match (opt_b_left, opt_b_right) {
            (Some(false), _) | (_, Some(false)) => Some(false),
            (Some(true), _) | (_, Some(true)) => Some(true),
            _ => None,
        }
    }
}
