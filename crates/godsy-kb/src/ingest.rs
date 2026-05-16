use std::path::{Path, PathBuf};
use std::sync::Arc;

use godsy_llm::{EmbeddingProvider, EmbeddingRequest};
use serde::{Deserialize, Serialize};

use crate::chunker::chunk_text;
use crate::error::{KbError, Result};
use crate::extractor::{extract_text, walk_supported, DocumentKind};
use crate::store::KbStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestReport {
    pub document_id: String,
    pub source_path: PathBuf,
    pub kind: String,
    pub chunks: usize,
}

#[derive(Debug)]
pub struct IngestService {
    store: Arc<KbStore>,
    embedder: Arc<dyn EmbeddingProvider>,
    embedding_model: String,
    chunk_size_chars: usize,
    chunk_overlap_chars: usize,
}

impl IngestService {
    pub fn new(
        store: Arc<KbStore>,
        embedder: Arc<dyn EmbeddingProvider>,
        embedding_model: impl Into<String>,
        chunk_size_chars: usize,
        chunk_overlap_chars: usize,
    ) -> Self {
        Self {
            store,
            embedder,
            embedding_model: embedding_model.into(),
            chunk_size_chars,
            chunk_overlap_chars,
        }
    }

    pub async fn ingest_path(&self, path: &Path) -> Result<Vec<IngestReport>> {
        let mut reports = Vec::new();
        let files = walk_supported(path)?;
        if files.is_empty() {
            return Err(KbError::Invalid(format!(
                "no supported files found under {}",
                path.display()
            )));
        }
        for file in files {
            let report = self.ingest_file(&file).await?;
            reports.push(report);
        }
        Ok(reports)
    }

    pub async fn ingest_file(&self, path: &Path) -> Result<IngestReport> {
        let (kind, body) = extract_text(path)?;
        let chunks = chunk_text(&body, self.chunk_size_chars, self.chunk_overlap_chars);
        if chunks.is_empty() {
            return Err(KbError::Invalid(format!("no extractable text in {}", path.display())));
        }
        let mut prepared = Vec::with_capacity(chunks.len());
        for c in chunks {
            let resp = self
                .embedder
                .embed(EmbeddingRequest {
                    model: self.embedding_model.clone(),
                    input: c.text.clone(),
                })
                .await?;
            prepared.push((c.ordinal, c.text, resp.vector));
        }
        let title = path.file_name().and_then(|s| s.to_str()).unwrap_or("(untitled)").to_string();
        let source = path.to_string_lossy().to_string();
        let document_id =
            self.store.upsert_document(&source, &title, kind_label(kind), &prepared)?;
        Ok(IngestReport {
            document_id,
            source_path: path.to_path_buf(),
            kind: kind_label(kind).to_string(),
            chunks: prepared.len(),
        })
    }
}

fn kind_label(k: DocumentKind) -> &'static str {
    match k {
        DocumentKind::Text => "text",
        DocumentKind::Markdown => "markdown",
        DocumentKind::Pdf => "pdf",
        DocumentKind::Docx => "docx",
        DocumentKind::Xlsx => "xlsx",
    }
}
