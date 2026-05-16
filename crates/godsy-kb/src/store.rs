use std::path::{Path, PathBuf};
use std::sync::Mutex;

use godsy_llm::cosine_similarity;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredChunk {
    pub id: String,
    pub document_id: String,
    pub document_path: String,
    pub document_title: String,
    pub ordinal: u32,
    pub text: String,
    pub score: f32,
}

/// SQLite-backed chunk + vector store. Vectors are persisted as little-endian
/// f32 BLOBs; cosine similarity is computed in-process at query time. This
/// keeps the dependency surface to one bundled SQLite without requiring the
/// `sqlite-vec` extension to be loaded out-of-band — which fails silently on
/// many Windows distributions.
#[derive(Debug)]
pub struct KbStore {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl KbStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn: Mutex::new(conn), path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Replace any prior ingest of the same `source_path` with a fresh document
    /// row + chunk rows. Returns the new document id.
    pub fn upsert_document(
        &self,
        source_path: &str,
        title: &str,
        kind: &str,
        chunks: &[(u32, String, Vec<f32>)],
    ) -> Result<String> {
        let mut conn = self.conn.lock().expect("kb store poisoned");
        let tx = conn.transaction()?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT id FROM documents WHERE source_path = ?1",
                params![source_path],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(prev) = existing {
            tx.execute("DELETE FROM chunks WHERE document_id = ?1", params![prev])?;
            tx.execute("DELETE FROM documents WHERE id = ?1", params![prev])?;
        }
        let doc_id = Uuid::new_v4().to_string();
        let ts = OffsetDateTime::now_utc().unix_timestamp();
        tx.execute(
            "INSERT INTO documents(id, source_path, title, kind, ingested_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![doc_id, source_path, title, kind, ts],
        )?;
        for (ordinal, text, vec) in chunks {
            let id = format!("kb-{}", Uuid::new_v4().simple());
            let bytes = vec_to_bytes(vec);
            tx.execute(
                "INSERT INTO chunks(id, document_id, ordinal, text, embedding, dim) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    doc_id,
                    ordinal,
                    text,
                    bytes,
                    i64::try_from(vec.len()).unwrap_or(i64::MAX)
                ],
            )?;
        }
        tx.commit()?;
        Ok(doc_id)
    }

    pub fn document_count(&self) -> Result<u64> {
        let conn = self.conn.lock().expect("kb store poisoned");
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    pub fn chunk_count(&self) -> Result<u64> {
        let conn = self.conn.lock().expect("kb store poisoned");
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    pub fn list_documents(&self) -> Result<Vec<DocumentRow>> {
        let conn = self.conn.lock().expect("kb store poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, source_path, title, kind, ingested_at FROM documents ORDER BY ingested_at DESC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(DocumentRow {
                    id: r.get(0)?,
                    source_path: r.get(1)?,
                    title: r.get(2)?,
                    kind: r.get(3)?,
                    ingested_at: r.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn delete_document(&self, document_id: &str) -> Result<bool> {
        let conn = self.conn.lock().expect("kb store poisoned");
        conn.execute("DELETE FROM chunks WHERE document_id = ?1", params![document_id])?;
        let n = conn.execute("DELETE FROM documents WHERE id = ?1", params![document_id])?;
        Ok(n > 0)
    }

    /// Return the top-k chunks ranked by cosine similarity against `query_vec`.
    /// Cosine is computed over the in-process candidate set; for the workloads
    /// this app targets (operator KBs of hundreds-to-low-thousands of chunks)
    /// linear scan is fine and avoids extension dependencies.
    pub fn search(&self, query_vec: &[f32], top_k: usize) -> Result<Vec<StoredChunk>> {
        if top_k == 0 {
            return Ok(Vec::new());
        }
        let conn = self.conn.lock().expect("kb store poisoned");
        let mut stmt = conn.prepare(
            "SELECT c.id, c.document_id, d.source_path, d.title, c.ordinal, c.text, c.embedding \
             FROM chunks c JOIN documents d ON d.id = c.document_id",
        )?;
        let mut all: Vec<StoredChunk> = stmt
            .query_map([], |r| {
                let blob: Vec<u8> = r.get(6)?;
                let v = bytes_to_vec(&blob);
                let score = cosine_similarity(query_vec, &v);
                Ok(StoredChunk {
                    id: r.get(0)?,
                    document_id: r.get(1)?,
                    document_path: r.get(2)?,
                    document_title: r.get(3)?,
                    ordinal: r.get::<_, i64>(4)? as u32,
                    text: r.get(5)?,
                    score,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(top_k);
        Ok(all)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRow {
    pub id: String,
    pub source_path: String,
    pub title: String,
    pub kind: String,
    pub ingested_at: i64,
}

fn vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for f in v {
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

fn bytes_to_vec(b: &[u8]) -> Vec<f32> {
    let mut out = Vec::with_capacity(b.len() / 4);
    let mut i = 0;
    while i + 4 <= b.len() {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&b[i..i + 4]);
        out.push(f32::from_le_bytes(buf));
        i += 4;
    }
    out
}

const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY,
    source_path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    kind TEXT NOT NULL,
    ingested_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS chunks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id),
    ordinal INTEGER NOT NULL,
    text TEXT NOT NULL,
    embedding BLOB NOT NULL,
    dim INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS chunks_doc_idx ON chunks(document_id);
";

impl KbStore {
    /// Convenience helper used by the Tauri layer to surface DB-level error
    /// strings without leaking the full `KbError` enum.
    pub fn describe(&self) -> Result<String> {
        Ok(format!(
            "kb at {} ({} docs, {} chunks)",
            self.path.display(),
            self.document_count()?,
            self.chunk_count()?
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        std::env::temp_dir().join(format!("godsy-kb-store-{}.sqlite", Uuid::new_v4()))
    }

    #[test]
    fn upsert_and_search_returns_sorted_top_k() {
        let p = tmp();
        let store = KbStore::open(&p).unwrap();
        let chunks = vec![
            (0_u32, "alpha".to_string(), vec![1.0_f32, 0.0]),
            (1_u32, "beta".to_string(), vec![0.0_f32, 1.0]),
            (2_u32, "gamma".to_string(), vec![0.7_f32, 0.7]),
        ];
        store.upsert_document("a.txt", "a", "text", &chunks).unwrap();
        let hits = store.search(&[1.0, 0.0], 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].text, "alpha");
        assert!(hits[0].score > hits[1].score);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn upsert_replaces_prior_ingest_of_same_path() {
        let p = tmp();
        let store = KbStore::open(&p).unwrap();
        let v1 = vec![(0_u32, "x".to_string(), vec![1.0, 0.0])];
        store.upsert_document("z.txt", "z", "text", &v1).unwrap();
        let v2 = vec![
            (0_u32, "y1".to_string(), vec![1.0, 0.0]),
            (1_u32, "y2".to_string(), vec![0.0, 1.0]),
        ];
        store.upsert_document("z.txt", "z", "text", &v2).unwrap();
        assert_eq!(store.document_count().unwrap(), 1);
        assert_eq!(store.chunk_count().unwrap(), 2);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn empty_store_search_is_empty() {
        let p = tmp();
        let store = KbStore::open(&p).unwrap();
        let hits = store.search(&[1.0, 0.0], 5).unwrap();
        assert!(hits.is_empty());
        std::fs::remove_file(&p).ok();
    }
}
