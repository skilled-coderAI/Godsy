use godsy_core::config::GodsyConfig as CoreConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;

// ─── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub provider: String,
    pub model: String,
    pub model_url: String,
    pub api_key: String,
    pub grounding: String,
    pub grounding_url: String,
    pub out_dir: String,
    pub confidence_threshold: f64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".into(),
            model: "qwen2.5".into(),
            model_url: "http://localhost:11434".into(),
            api_key: String::new(),
            grounding: "none".into(),
            grounding_url: String::new(),
            out_dir: "./godsy-plans".into(),
            confidence_threshold: 0.75,
        }
    }
}

fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("godsy.toml")
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> UiConfig {
    let path = config_path(&app);
    if path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<CoreConfig>(&raw) {
                return UiConfig {
                    provider: format!("{:?}", cfg.model.provider).to_lowercase(),
                    model: cfg.model.model_name,
                    model_url: cfg.model.base_url,
                    api_key: cfg.model.api_key,
                    grounding: format!("{:?}", cfg.grounding.kind).to_lowercase(),
                    grounding_url: cfg.grounding.base_url.unwrap_or_default(),
                    out_dir: cfg.out_dir,
                    confidence_threshold: cfg.confidence_threshold,
                };
            }
        }
    }
    UiConfig::default()
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: UiConfig) -> Result<(), String> {
    let path = config_path(&app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Build a minimal TOML representation
    let toml_content = format!(
        r#"[model]
provider = "{provider}"
model_name = "{model}"
base_url = "{model_url}"
api_key = "{api_key}"

[grounding]
kind = "{grounding}"
{grounding_url_line}

out_dir = "{out_dir}"
confidence_threshold = {confidence_threshold}
"#,
        provider = config.provider,
        model = config.model,
        model_url = config.model_url,
        api_key = config.api_key,
        grounding = config.grounding,
        grounding_url_line = if config.grounding_url.is_empty() {
            String::new()
        } else {
            format!("base_url = \"{}\"", config.grounding_url)
        },
        out_dir = config.out_dir,
        confidence_threshold = config.confidence_threshold,
    );

    std::fs::write(&path, toml_content).map_err(|e| e.to_string())
}

// ─── Plan History ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub status: String,
    pub out_dir: String,
}

#[tauri::command]
pub fn list_plans(app: AppHandle) -> Vec<PlanSummary> {
    let base = config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("plans");

    let Ok(entries) = std::fs::read_dir(&base) else {
        return vec![];
    };

    let mut plans: Vec<PlanSummary> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let dir = e.path();
            let id = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let prd_path = dir.join("PRD.md");
            let title = if prd_path.exists() {
                std::fs::read_to_string(&prd_path)
                    .ok()
                    .and_then(|s| {
                        s.lines()
                            .find(|l| l.starts_with("# "))
                            .map(|l| l.trim_start_matches("# ").to_string())
                    })
                    .unwrap_or_else(|| id.clone())
            } else {
                id.clone()
            };

            PlanSummary {
                id: id.clone(),
                title,
                created_at: id.clone(),
                status: "complete".into(),
                out_dir: dir.to_string_lossy().into_owned(),
            }
        })
        .collect();

    plans.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    plans
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanContent {
    pub prd: Option<String>,
    pub api: Option<String>,
    pub ui: Option<String>,
    pub tasks: Option<String>,
    pub risks: Option<String>,
    pub prompt: Option<String>,
}

#[tauri::command]
pub fn get_plan(out_dir: String) -> PlanContent {
    let base = PathBuf::from(&out_dir);
    let read = |name: &str| std::fs::read_to_string(base.join(name)).ok();
    PlanContent {
        prd: read("PRD.md"),
        api: read("API.md"),
        ui: read("UI.md"),
        tasks: read("tasks.json"),
        risks: read("risks.md"),
        prompt: read("CODING_AGENT_PROMPT.md"),
    }
}

// ─── Run Plan ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProgress {
    pub agent: String,
    pub status: String, // "running" | "done" | "failed"
    pub message: Option<String>,
}

#[tauri::command]
pub async fn run_plan(
    app: AppHandle,
    request: String,
    window: tauri::Window,
) -> Result<String, String> {
    let config = get_config(app);

    // Emit pipeline start event
    window
        .emit(
            "plan:progress",
            AgentProgress {
                agent: "product_manager".into(),
                status: "running".into(),
                message: None,
            },
        )
        .map_err(|e| e.to_string())?;

    // TODO: wire godsy-agents orchestrator here
    // For now emit a stub sequence to demonstrate the UI
    let agents = [
        "product_manager",
        "researcher",
        "architect",
        "api_designer",
        "ui_designer",
        "tech_lead",
        "estimator",
        "risk_reviewer",
        "validator",
    ];

    let window_clone = window.clone();
    let agents_owned: Vec<String> = agents.iter().map(|s| s.to_string()).collect();
    let _request = request.clone();
    let _config = config;

    tokio::spawn(async move {
        for agent in &agents_owned {
            let _ = window_clone.emit(
                "plan:progress",
                AgentProgress {
                    agent: agent.clone(),
                    status: "running".into(),
                    message: Some(format!("{} is analysing your request…", agent)),
                },
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
            let _ = window_clone.emit(
                "plan:progress",
                AgentProgress {
                    agent: agent.clone(),
                    status: "done".into(),
                    message: Some(format!("{} completed.", agent)),
                },
            );
        }
        let _ = window_clone.emit("plan:complete", ());
    });

    Ok("Planning pipeline started".into())
}

// ─── Knowledge Base ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KbFileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub file_type: String,
    pub added_at: String,
}

#[tauri::command]
pub fn list_kb_files(app: AppHandle) -> Vec<KbFileInfo> {
    let base = config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("kb");

    let Ok(entries) = std::fs::read_dir(&base) else {
        return vec![];
    };

    entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| {
            let path = e.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            let ext = path
                .extension()
                .map(|x| x.to_string_lossy().to_uppercase())
                .unwrap_or_else(|| "FILE".into());
            KbFileInfo {
                id: name.clone(),
                name,
                size,
                file_type: ext.into_owned(),
                added_at: String::new(),
            }
        })
        .collect()
}

#[tauri::command]
pub fn upload_kb_file(app: AppHandle, path: String) -> Result<(), String> {
    let kb_dir = config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("kb");
    std::fs::create_dir_all(&kb_dir).map_err(|e| e.to_string())?;

    let source = PathBuf::from(&path);
    let file_name = source
        .file_name()
        .ok_or_else(|| "No file name in path".to_string())?;
    std::fs::copy(&source, kb_dir.join(file_name)).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_kb_file(app: AppHandle, id: String) -> Result<(), String> {
    let path = config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("kb")
        .join(&id);
    std::fs::remove_file(path).map_err(|e| e.to_string())
}
