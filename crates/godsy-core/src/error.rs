use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("plan validation failed: {0}")]
    Validation(String),

    #[error("missing citation for claim: {0}")]
    MissingCitation(String),

    #[error("dangling reference: {0}")]
    DanglingReference(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
