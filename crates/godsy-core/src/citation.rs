use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CitationKind {
    Web { url: String },
    KnowledgeBase { chunk_id: String },
    ProjectFile { path: String, line: Option<u32> },
    User { message_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    pub source: CitationKind,
    pub snippet: String,
    #[serde(with = "time::serde::rfc3339")]
    pub retrieved_at: OffsetDateTime,
}

impl Citation {
    pub fn new(id: impl Into<String>, source: CitationKind, snippet: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source,
            snippet: snippet.into(),
            retrieved_at: OffsetDateTime::now_utc(),
        }
    }
}
