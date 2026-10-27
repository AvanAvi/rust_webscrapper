//! Lists the purposes a crawler can declare.

use serde::{Deserialize, Serialize};

/// Use the crawled content is collected for; `AiTrain` is the strictest and the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    /// Building a search index.
    Search,
    /// Feeding content to a model at inference time.
    AiInput,
    /// Training or fine-tuning a model.
    #[default]
    AiTrain,
    /// Long-term archiving.
    Archive,
}

impl Purpose {
    /// Returns a short phrase naming the purpose in plain language.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Search => "search indexing",
            Self::AiInput => "AI input (using content at inference time)",
            Self::AiTrain => "AI training",
            Self::Archive => "archiving",
        }
    }

    /// Returns the stable machine name, as used in JSON and the User-Agent.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::AiInput => "ai_input",
            Self::AiTrain => "ai_train",
            Self::Archive => "archive",
        }
    }

    /// Returns whether the purpose is an AI use that TDM and AI opt-outs cover.
    pub fn is_ai_use(&self) -> bool {
        matches!(self, Self::AiInput | Self::AiTrain)
    }
}
