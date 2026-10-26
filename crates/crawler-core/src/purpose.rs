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
