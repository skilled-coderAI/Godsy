use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use godsy_agents::{AgentContext, ExplainRecorder, Orchestrator, OrchestratorConfig};
use godsy_core::{GodsyConfig, GroundingKind, PlanBundleWriter, ProviderKind};
use godsy_grounding::{GroundingProvider, MultiGrounder, NoopGrounder, VaneGrounder, VaneSettings};
use godsy_kb::{IngestService, KbGrounder, KbStore};
use godsy_llm::{
    CloudflareProvider, EmbeddingProvider, EmbeddingRequest, LlmProvider, OllamaEmbedder,
    OllamaProvider,
};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "godsy", version, about = "Godsy planning studio")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Write a default `godsy.toml` next to the binary working directory.
    Init {
        #[arg(long, default_value = "godsy.toml")]
        config: PathBuf,
    },
    /// Run the planning team on a business request and write a Plan Bundle.
    Plan(Box<PlanArgs>),
    /// Manage the local knowledge base (PDF/DOCX/XLSX/MD/TXT/...).
    #[command(subcommand)]
    Kb(KbCmd),
}

#[derive(clap::Args, Debug)]
struct PlanArgs {
    request: String,
    #[arg(long, default_value = "godsy.toml")]
    config: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long, alias = "ollama-url")]
    model_url: Option<String>,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    api_key: Option<String>,
    #[arg(long)]
    cloudflare_account_id: Option<String>,
    #[arg(long)]
    grounding_url: Option<String>,
    #[arg(long)]
    grounding: Option<String>,
    /// Force-include the local KB grounder regardless of `knowledge_base.use_in_planning`.
    #[arg(long)]
    use_kb: bool,
    #[arg(long)]
    explain: bool,
    #[arg(long)]
    strict: bool,
}

#[derive(Subcommand, Debug)]
enum KbCmd {
    /// Ingest a file or directory into the KB. Recursively walks directories.
    Add {
        path: PathBuf,
        #[arg(long, default_value = "godsy.toml")]
        config: PathBuf,
    },
    /// List documents currently stored in the KB.
    List {
        #[arg(long, default_value = "godsy.toml")]
        config: PathBuf,
    },
    /// Run a semantic search against the KB and print the top hits.
    Search {
        query: String,
        #[arg(long, default_value = "godsy.toml")]
        config: PathBuf,
        #[arg(long)]
        top_k: Option<usize>,
    },
    /// Remove a document and all its chunks by document id.
    Remove {
        document_id: String,
        #[arg(long, default_value = "godsy.toml")]
        config: PathBuf,
    },
    /// Print a one-line summary of the KB store.
    Status {
        #[arg(long, default_value = "godsy.toml")]
        config: PathBuf,
    },
}

fn load_or_init(config: &std::path::Path) -> Result<GodsyConfig> {
    if config.exists() {
        GodsyConfig::load(config).with_context(|| format!("loading config {}", config.display()))
    } else {
        GodsyConfig::write_default(config)?;
        GodsyConfig::load(config).map_err(Into::into)
    }
}

fn build_llm_provider(cfg: &GodsyConfig) -> Result<Arc<dyn LlmProvider>> {
    let timeout = std::time::Duration::from_secs(cfg.model.request_timeout_secs);
    let provider: Arc<dyn LlmProvider> = match cfg.model.provider {
        ProviderKind::Ollama | ProviderKind::OllamaCloud => Arc::new(OllamaProvider::with_api_key(
            cfg.model.base_url.clone(),
            cfg.model.api_key.clone(),
            timeout,
        )),
        ProviderKind::CloudflareWorkers => Arc::new(CloudflareProvider::new(
            cfg.model.base_url.clone(),
            cfg.model.cloudflare_account_id.clone(),
            cfg.model.api_key.clone(),
            timeout,
        )),
    };
    Ok(provider)
}

fn build_embedder(cfg: &GodsyConfig) -> Arc<OllamaEmbedder> {
    Arc::new(OllamaEmbedder::with_api_key(
        cfg.embedding.base_url.clone(),
        cfg.model.api_key.clone(),
        std::time::Duration::from_secs(cfg.embedding.request_timeout_secs),
    ))
}

fn build_grounder(cfg: &GodsyConfig, force_kb: bool) -> Result<Arc<dyn GroundingProvider>> {
    let mut providers: Vec<Arc<dyn GroundingProvider>> = Vec::new();
    match cfg.grounding.provider {
        GroundingKind::None => {}
        GroundingKind::Vane => {
            let mut settings = VaneSettings::local_ollama(cfg.grounding.base_url.clone());
            if let Some(vc) = &cfg.grounding.vane {
                settings.focus_mode.clone_from(&vc.focus_mode);
                settings.optimization_mode.clone_from(&vc.optimization_mode);
                settings.chat_provider.clone_from(&vc.chat_provider);
                settings.chat_model.clone_from(&vc.chat_model);
                settings.embedding_provider.clone_from(&vc.embedding_provider);
                settings.embedding_model.clone_from(&vc.embedding_model);
            }
            providers.push(Arc::new(VaneGrounder::new(
                settings,
                std::time::Duration::from_secs(cfg.grounding.request_timeout_secs),
            )));
        }
    }
    if force_kb || cfg.knowledge_base.use_in_planning {
        let store = Arc::new(KbStore::open(&cfg.knowledge_base.db_path)?);
        let embedder = build_embedder(cfg);
        providers.push(Arc::new(KbGrounder::new(
            store,
            embedder,
            cfg.embedding.model.clone(),
            cfg.knowledge_base.top_k,
        )));
    }
    Ok(match providers.len() {
        0 => Arc::new(NoopGrounder),
        1 => providers.remove(0),
        _ => Arc::new(MultiGrounder::new(providers)),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { config } => {
            GodsyConfig::write_default(&config)
                .with_context(|| format!("writing default config to {}", config.display()))?;
            println!("wrote default config: {}", config.display());
        }
        Cmd::Plan(args) => run_plan(*args).await?,
        Cmd::Kb(kb) => run_kb(kb).await?,
    }
    Ok(())
}

async fn run_plan(args: PlanArgs) -> Result<()> {
    let PlanArgs {
        request,
        config,
        out,
        model,
        model_url,
        provider,
        api_key,
        cloudflare_account_id,
        grounding_url,
        grounding,
        use_kb,
        explain,
        strict,
    } = args;
    let mut cfg = load_or_init(&config)?;
    if let Some(p) = provider {
        cfg.model.provider = match p.as_str() {
            "ollama" => ProviderKind::Ollama,
            "ollama_cloud" | "ollama-cloud" => ProviderKind::OllamaCloud,
            "cloudflare_workers" | "cloudflare-workers" => ProviderKind::CloudflareWorkers,
            other => anyhow::bail!(
                "unknown --provider {other:?}: expected ollama|ollama_cloud|cloudflare_workers"
            ),
        };
    }
    if let Some(m) = model {
        cfg.model.model = m;
    }
    if let Some(u) = model_url {
        cfg.model.base_url = u;
    }
    if let Some(k) = api_key {
        cfg.model.api_key = k;
    }
    if let Some(a) = cloudflare_account_id {
        cfg.model.cloudflare_account_id = a;
    }
    if let Some(o) = out {
        cfg.output.out_dir = o;
    }
    if let Some(g) = grounding {
        cfg.grounding.provider = match g.as_str() {
            "none" => GroundingKind::None,
            "vane" => GroundingKind::Vane,
            other => anyhow::bail!("unknown --grounding {other:?}: expected none|vane"),
        };
    }
    if let Some(u) = grounding_url {
        cfg.grounding.base_url = u;
    }
    let cfg = revalidate(cfg)?;

    std::fs::create_dir_all(&cfg.output.out_dir)?;

    let provider = build_llm_provider(&cfg)?;
    let grounder = build_grounder(&cfg, use_kb)?;

    let recorder = if explain { Some(Arc::new(ExplainRecorder::default())) } else { None };
    let mut ctx = AgentContext::new(provider, cfg.model.model.clone()).with_grounder(grounder);
    if let Some(r) = recorder.clone() {
        ctx = ctx.with_explain(r);
    }
    let orch = Orchestrator::new(OrchestratorConfig {
        max_validator_retries: cfg.orchestrator.max_validator_retries,
    });
    let outcome = orch.run(&ctx, request).await?;
    let dir = PlanBundleWriter::new(&outcome.plan).write_to(&cfg.output.out_dir)?;

    if let Some(r) = recorder {
        let path = dir.join("explain.jsonl");
        let mut buf = String::new();
        for ev in r.drain() {
            buf.push_str(&serde_json::to_string(&ev)?);
            buf.push('\n');
        }
        std::fs::write(&path, buf)?;
        println!("explain trace: {}", path.display());
    }

    println!("plan written to: {}", dir.display());
    println!("validator passed: {}", outcome.validator_passed);
    println!("retries: {}", outcome.retries);

    if strict && !outcome.validator_passed {
        anyhow::bail!("validator did not pass within retry budget (--strict)");
    }
    Ok(())
}

fn revalidate(cfg: GodsyConfig) -> Result<GodsyConfig> {
    let raw = toml::to_string(&cfg)?;
    let tmp = std::env::temp_dir().join(format!("godsy-effective-{}.toml", std::process::id()));
    std::fs::write(&tmp, raw)?;
    let loaded = GodsyConfig::load(&tmp)?;
    std::fs::remove_file(&tmp).ok();
    Ok(loaded)
}

async fn run_kb(cmd: KbCmd) -> Result<()> {
    match cmd {
        KbCmd::Add { path, config } => {
            let cfg = load_or_init(&config)?;
            let store = Arc::new(KbStore::open(&cfg.knowledge_base.db_path)?);
            let embedder = build_embedder(&cfg);
            let svc = IngestService::new(
                store.clone(),
                embedder,
                cfg.embedding.model.clone(),
                cfg.knowledge_base.chunk_size_chars,
                cfg.knowledge_base.chunk_overlap_chars,
            );
            let reports = svc.ingest_path(&path).await?;
            for r in &reports {
                println!(
                    "ingested {} ({}) -> {} chunks  doc_id={}",
                    r.source_path.display(),
                    r.kind,
                    r.chunks,
                    r.document_id
                );
            }
            println!("done: {} document(s)", reports.len());
        }
        KbCmd::List { config } => {
            let cfg = load_or_init(&config)?;
            let store = KbStore::open(&cfg.knowledge_base.db_path)?;
            let docs = store.list_documents()?;
            if docs.is_empty() {
                println!("(empty kb at {})", store.path().display());
            } else {
                for d in docs {
                    println!("{}  {}  {}  ({})", d.id, d.kind, d.title, d.source_path);
                }
            }
        }
        KbCmd::Search { query, config, top_k } => {
            let cfg = load_or_init(&config)?;
            let store = Arc::new(KbStore::open(&cfg.knowledge_base.db_path)?);
            let embedder = build_embedder(&cfg);
            let k = top_k.unwrap_or(cfg.knowledge_base.top_k);
            let resp = embedder
                .embed(EmbeddingRequest { model: cfg.embedding.model.clone(), input: query })
                .await
                .context("embedding query")?;
            let hits = store.search(&resp.vector, k)?;
            if hits.is_empty() {
                println!("(no hits)");
            } else {
                for h in hits {
                    println!(
                        "[{:.3}] {} #{}  {}",
                        h.score, h.document_title, h.ordinal, h.document_path
                    );
                    let preview: String = h.text.chars().take(160).collect();
                    println!("   {preview}");
                }
            }
        }
        KbCmd::Remove { document_id, config } => {
            let cfg = load_or_init(&config)?;
            let store = KbStore::open(&cfg.knowledge_base.db_path)?;
            let removed = store.delete_document(&document_id)?;
            println!("{}", if removed { "removed" } else { "no such document" });
        }
        KbCmd::Status { config } => {
            let cfg = load_or_init(&config)?;
            let store = KbStore::open(&cfg.knowledge_base.db_path)?;
            println!("{}", store.describe()?);
        }
    }
    Ok(())
}
