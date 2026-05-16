use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ProviderKind {
    Ollama,
    OllamaCloud,
    CloudflareWorkers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    pub provider: ProviderKind,
    pub base_url: String,
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    /// API key for hosted Ollama (`ollama_cloud`) or any other provider that
    /// requires bearer-token authentication. Optional; when empty no
    /// `Authorization` header is sent. May be overridden by `OLLAMA_API_KEY`
    /// or `GODSY_API_KEY` environment variables.
    #[serde(default)]
    pub api_key: String,
    /// Cloudflare account id, only used when `provider = "cloudflare_workers"`.
    /// Override via `CLOUDFLARE_ACCOUNT_ID`.
    #[serde(default)]
    pub cloudflare_account_id: String,
}

const fn default_temperature() -> f32 {
    0.2
}
const fn default_request_timeout_secs() -> u64 {
    180
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrchestratorConfig {
    #[serde(default = "default_retries")]
    pub max_validator_retries: u32,
    #[serde(default = "default_threshold")]
    pub confidence_threshold: f32,
}

const fn default_retries() -> u32 {
    1
}
const fn default_threshold() -> f32 {
    0.8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    pub out_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum GroundingKind {
    None,
    Vane,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundingConfig {
    pub provider: GroundingKind,
    #[serde(default)]
    pub base_url: String,
    #[serde(default = "default_max_hits")]
    pub max_hits: usize,
    #[serde(default = "default_grounding_timeout_secs")]
    pub request_timeout_secs: u64,
    #[serde(default)]
    pub vane: Option<VaneConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaneConfig {
    #[serde(default = "default_vane_focus")]
    pub focus_mode: String,
    #[serde(default = "default_vane_optimization")]
    pub optimization_mode: String,
    #[serde(default = "default_vane_chat_provider")]
    pub chat_provider: String,
    #[serde(default = "default_vane_chat_model")]
    pub chat_model: String,
    #[serde(default = "default_vane_embedding_provider")]
    pub embedding_provider: String,
    #[serde(default = "default_vane_embedding_model")]
    pub embedding_model: String,
}

const fn default_max_hits() -> usize {
    6
}
const fn default_grounding_timeout_secs() -> u64 {
    60
}
fn default_vane_focus() -> String {
    "webSearch".to_string()
}
fn default_vane_optimization() -> String {
    "balanced".to_string()
}
fn default_vane_chat_provider() -> String {
    "ollama".to_string()
}
fn default_vane_chat_model() -> String {
    "llama3.1".to_string()
}
fn default_vane_embedding_provider() -> String {
    "ollama".to_string()
}
fn default_vane_embedding_model() -> String {
    "nomic-embed-text".to_string()
}

impl Default for GroundingConfig {
    fn default() -> Self {
        Self {
            provider: GroundingKind::None,
            base_url: String::new(),
            max_hits: default_max_hits(),
            request_timeout_secs: default_grounding_timeout_secs(),
            vane: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingConfig {
    /// `ollama` is the only supported embedding backend today; reserved as an
    /// enum-shaped string for forward-compat with cloud embedders.
    #[serde(default = "default_embedding_provider")]
    pub provider: String,
    #[serde(default = "default_embedding_base_url")]
    pub base_url: String,
    #[serde(default = "default_embedding_model")]
    pub model: String,
    #[serde(default = "default_embedding_timeout_secs")]
    pub request_timeout_secs: u64,
}

fn default_embedding_provider() -> String {
    "ollama".to_string()
}
fn default_embedding_base_url() -> String {
    "http://localhost:11434".to_string()
}
fn default_embedding_model() -> String {
    "nomic-embed-text".to_string()
}
const fn default_embedding_timeout_secs() -> u64 {
    60
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: default_embedding_provider(),
            base_url: default_embedding_base_url(),
            model: default_embedding_model(),
            request_timeout_secs: default_embedding_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeBaseConfig {
    /// SQLite file backing the KB. Created on first ingest.
    #[serde(default = "default_kb_path")]
    pub db_path: PathBuf,
    #[serde(default = "default_kb_chunk_size")]
    pub chunk_size_chars: usize,
    #[serde(default = "default_kb_chunk_overlap")]
    pub chunk_overlap_chars: usize,
    /// How many chunks the KB grounder returns per query.
    #[serde(default = "default_kb_top_k")]
    pub top_k: usize,
    /// When true, the planning pipeline calls the KB grounder *in addition to*
    /// the configured web grounder. Hits are merged before the LLM step.
    #[serde(default)]
    pub use_in_planning: bool,
}

fn default_kb_path() -> PathBuf {
    PathBuf::from("godsy-kb.sqlite")
}
const fn default_kb_chunk_size() -> usize {
    1200
}
const fn default_kb_chunk_overlap() -> usize {
    150
}
const fn default_kb_top_k() -> usize {
    5
}

impl Default for KnowledgeBaseConfig {
    fn default() -> Self {
        Self {
            db_path: default_kb_path(),
            chunk_size_chars: default_kb_chunk_size(),
            chunk_overlap_chars: default_kb_chunk_overlap(),
            top_k: default_kb_top_k(),
            use_in_planning: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GodsyConfig {
    pub model: ModelConfig,
    pub orchestrator: OrchestratorConfig,
    pub output: OutputConfig,
    #[serde(default)]
    pub grounding: GroundingConfig,
    #[serde(default)]
    pub embedding: EmbeddingConfig,
    #[serde(default)]
    pub knowledge_base: KnowledgeBaseConfig,
}

impl GodsyConfig {
    /// Load configuration from a TOML file at `path`. Environment variables
    /// `GODSY_MODEL`, `GODSY_MODEL_BASE_URL`, `GODSY_OUT_DIR`,
    /// `GODSY_GROUNDING_URL`, `OLLAMA_API_KEY` / `GODSY_API_KEY`,
    /// `CLOUDFLARE_ACCOUNT_ID` override their respective fields when set.
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| CoreError::Validation(format!("config read {}: {e}", path.display())))?;
        let mut cfg: Self = toml::from_str(&raw)
            .map_err(|e| CoreError::Validation(format!("config parse {}: {e}", path.display())))?;
        cfg.apply_env_overrides();
        cfg.validate()?;
        Ok(cfg)
    }

    /// Write the default configuration to `path`. Used by `godsy init` when no
    /// config exists yet, so the user gets a real, editable file rather than
    /// implicit hidden defaults.
    pub fn write_default(path: &Path) -> Result<()> {
        let cfg = Self {
            model: ModelConfig {
                provider: ProviderKind::Ollama,
                base_url: "http://localhost:11434".to_string(),
                model: "qwen2.5".to_string(),
                temperature: default_temperature(),
                request_timeout_secs: default_request_timeout_secs(),
                api_key: String::new(),
                cloudflare_account_id: String::new(),
            },
            orchestrator: OrchestratorConfig {
                max_validator_retries: default_retries(),
                confidence_threshold: default_threshold(),
            },
            output: OutputConfig { out_dir: PathBuf::from("plans-out") },
            grounding: GroundingConfig::default(),
            embedding: EmbeddingConfig::default(),
            knowledge_base: KnowledgeBaseConfig::default(),
        };
        let text = toml::to_string_pretty(&cfg)
            .map_err(|e| CoreError::Validation(format!("config serialize: {e}")))?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, text)?;
        Ok(())
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("GODSY_MODEL") {
            self.model.model = v;
        }
        if let Ok(v) = std::env::var("GODSY_MODEL_BASE_URL") {
            self.model.base_url = v;
        }
        if let Ok(v) = std::env::var("GODSY_OUT_DIR") {
            self.output.out_dir = PathBuf::from(v);
        }
        if let Ok(v) = std::env::var("GODSY_GROUNDING_URL") {
            self.grounding.base_url = v;
        }
        if let Ok(v) = std::env::var("OLLAMA_API_KEY") {
            self.model.api_key = v;
        }
        if let Ok(v) = std::env::var("GODSY_API_KEY") {
            self.model.api_key = v;
        }
        if let Ok(v) = std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            self.model.cloudflare_account_id = v;
        }
    }

    fn validate(&self) -> Result<()> {
        if self.model.base_url.is_empty() {
            return Err(CoreError::Validation("model.base_url is empty".into()));
        }
        if self.model.model.is_empty() {
            return Err(CoreError::Validation("model.model is empty".into()));
        }
        if !(0.0..=2.0).contains(&self.model.temperature) {
            return Err(CoreError::Validation("model.temperature must be in [0, 2]".into()));
        }
        if matches!(self.model.provider, ProviderKind::OllamaCloud) && self.model.api_key.is_empty()
        {
            return Err(CoreError::Validation(
                "model.api_key is required when model.provider = \"ollama_cloud\" \
                 (set in godsy.toml, or via OLLAMA_API_KEY / GODSY_API_KEY)"
                    .into(),
            ));
        }
        if matches!(self.model.provider, ProviderKind::CloudflareWorkers) {
            if self.model.api_key.is_empty() {
                return Err(CoreError::Validation(
                    "model.api_key is required when model.provider = \"cloudflare_workers\" \
                     (Cloudflare API token; set in godsy.toml or GODSY_API_KEY)"
                        .into(),
                ));
            }
            if self.model.cloudflare_account_id.is_empty() {
                return Err(CoreError::Validation(
                    "model.cloudflare_account_id is required for cloudflare_workers \
                     (set in godsy.toml or CLOUDFLARE_ACCOUNT_ID)"
                        .into(),
                ));
            }
        }
        if !(0.0..=1.0).contains(&self.orchestrator.confidence_threshold) {
            return Err(CoreError::Validation(
                "orchestrator.confidence_threshold must be in [0, 1]".into(),
            ));
        }
        match self.grounding.provider {
            GroundingKind::None => {}
            GroundingKind::Vane => {
                if self.grounding.base_url.is_empty() {
                    return Err(CoreError::Validation(
                        "grounding.base_url must be set when grounding.provider = \"vane\"".into(),
                    ));
                }
            }
        }
        if self.grounding.max_hits == 0 {
            return Err(CoreError::Validation("grounding.max_hits must be > 0".into()));
        }
        if self.knowledge_base.chunk_size_chars == 0 {
            return Err(CoreError::Validation(
                "knowledge_base.chunk_size_chars must be > 0".into(),
            ));
        }
        if self.knowledge_base.chunk_overlap_chars >= self.knowledge_base.chunk_size_chars {
            return Err(CoreError::Validation(
                "knowledge_base.chunk_overlap_chars must be < chunk_size_chars".into(),
            ));
        }
        if self.knowledge_base.top_k == 0 {
            return Err(CoreError::Validation("knowledge_base.top_k must be > 0".into()));
        }
        if self.embedding.provider != "ollama" {
            return Err(CoreError::Validation(
                "embedding.provider only supports \"ollama\" today".into(),
            ));
        }
        if self.embedding.base_url.is_empty() || self.embedding.model.is_empty() {
            return Err(CoreError::Validation(
                "embedding.base_url and embedding.model must both be set".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_default() {
        let dir = std::env::temp_dir().join(format!("godsy-cfg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("godsy.toml");
        GodsyConfig::write_default(&path).unwrap();
        let cfg = GodsyConfig::load(&path).unwrap();
        assert_eq!(cfg.model.base_url, "http://localhost:11434");
        assert_eq!(cfg.model.model, "qwen2.5");
        assert_eq!(cfg.embedding.model, "nomic-embed-text");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_unknown_field() {
        let dir = std::env::temp_dir().join(format!("godsy-cfg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("godsy.toml");
        std::fs::write(
            &path,
            "[model]\nprovider = \"ollama\"\nbase_url = \"x\"\nmodel = \"y\"\nbogus_field = 1\n\
             [orchestrator]\n[output]\nout_dir = \"o\"\n",
        )
        .unwrap();
        assert!(GodsyConfig::load(&path).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_out_of_range_temperature() {
        let dir = std::env::temp_dir().join(format!("godsy-cfg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("godsy.toml");
        std::fs::write(
            &path,
            "[model]\nprovider=\"ollama\"\nbase_url=\"x\"\nmodel=\"y\"\ntemperature=9.0\n\
             [orchestrator]\n[output]\nout_dir=\"o\"\n",
        )
        .unwrap();
        assert!(GodsyConfig::load(&path).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_ollama_cloud_without_api_key() {
        let dir = std::env::temp_dir().join(format!("godsy-cfg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("godsy.toml");
        std::fs::write(
            &path,
            "[model]\nprovider=\"ollama_cloud\"\nbase_url=\"https://ollama.com\"\nmodel=\"y\"\n\
             [orchestrator]\n[output]\nout_dir=\"o\"\n",
        )
        .unwrap();
        std::env::remove_var("OLLAMA_API_KEY");
        std::env::remove_var("GODSY_API_KEY");
        assert!(GodsyConfig::load(&path).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_cloudflare_without_account_id() {
        let dir = std::env::temp_dir().join(format!("godsy-cfg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("godsy.toml");
        std::fs::write(
            &path,
            "[model]\nprovider=\"cloudflare_workers\"\nbase_url=\"https://api.cloudflare.com/client/v4\"\n\
             model=\"@cf/meta/llama-3.1-8b-instruct\"\napi_key=\"tok\"\n\
             [orchestrator]\n[output]\nout_dir=\"o\"\n",
        )
        .unwrap();
        std::env::remove_var("CLOUDFLARE_ACCOUNT_ID");
        assert!(GodsyConfig::load(&path).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_kb_overlap_ge_chunk() {
        let dir = std::env::temp_dir().join(format!("godsy-cfg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("godsy.toml");
        std::fs::write(
            &path,
            "[model]\nprovider=\"ollama\"\nbase_url=\"x\"\nmodel=\"y\"\n\
             [orchestrator]\n[output]\nout_dir=\"o\"\n\
             [knowledge_base]\nchunk_size_chars=100\nchunk_overlap_chars=200\n",
        )
        .unwrap();
        assert!(GodsyConfig::load(&path).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
