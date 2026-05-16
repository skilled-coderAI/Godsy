pub mod cloudflare;
pub mod embedding;
#[cfg(any(test, feature = "test-support"))]
pub mod mock;
pub mod ollama;
pub mod ollama_embed;
pub mod provider;

pub use cloudflare::CloudflareProvider;
pub use embedding::{
    cosine_similarity, EmbeddingError, EmbeddingProvider, EmbeddingRequest, EmbeddingResponse,
};
#[cfg(any(test, feature = "test-support"))]
pub use mock::MockProvider;
pub use ollama::OllamaProvider;
pub use ollama_embed::OllamaEmbedder;
pub use provider::{ChatMessage, ChatRequest, ChatResponse, LlmError, LlmProvider, Role};
