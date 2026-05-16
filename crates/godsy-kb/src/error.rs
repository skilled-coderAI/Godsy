use thiserror::Error;

#[derive(Debug, Error)]
pub enum KbError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("embedding error: {0}")]
    Embedding(#[from] godsy_llm::EmbeddingError),
    #[error("unsupported document kind: {0}")]
    Unsupported(String),
    #[error("text extraction failed: {0}")]
    Extract(String),
    #[error("invalid argument: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, KbError>;
