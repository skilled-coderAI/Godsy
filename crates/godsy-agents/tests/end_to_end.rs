use std::sync::Arc;

use godsy_agents::{AgentContext, Orchestrator, OrchestratorConfig};
use godsy_core::PlanBundleWriter;
use godsy_llm::{LlmProvider, MockProvider};

fn pm_response() -> &'static str {
    r#"{"summary":"Track which trucks delivered which orders today",
        "success_criteria":["Operator can record delivery in <30s"],
        "assumptions":["Drivers do not enter data themselves"],
        "clarifications":["How many trucks per day?"]}"#
}

fn researcher_response() -> &'static str {
    r#"{"findings":[
        {"id":"cit1","source_url":"https://sqlite.org","snippet":"SQLite is embedded"},
        {"id":"cit2","source_url":"https://tauri.app","snippet":"Tauri is a desktop framework"}
    ]}"#
}

fn architect_response() -> &'static str {
    r#"{
      "overview":"Single-binary desktop app with local SQLite store",
      "components":[
        {"id":"ui","name":"Web UI","responsibility":"capture deliveries"},
        {"id":"api","name":"Local HTTP API","responsibility":"persist deliveries"},
        {"id":"db","name":"SQLite store","responsibility":"persist deliveries"}
      ],
      "mermaid_diagram":"graph TD; ui-->api; api-->db",
      "data_model":[{"id":"delivery","name":"Delivery","fields":[
        {"name":"id","ty":"integer"},
        {"name":"truck","ty":"text"},
        {"name":"order_no","ty":"text"}
      ]}],
      "stack":{"language":"Rust","frameworks":["Tauri","axum"],"storage":"SQLite",
               "runtime_target":"Windows desktop","rationale":"single binary, local",
               "citation_ids":["cit1","cit2"]}
    }"#
}

fn api_designer_response() -> &'static str {
    r#"{
      "base_url":"http://localhost:8080",
      "auth":null,
      "endpoints":[
        {"id":"e_create","method":"POST","path":"/deliveries","summary":"create",
         "auth_required":false,
         "path_params":[],"query_params":[],
         "request_body":{"content_type":"application/json",
                         "fields":[{"name":"truck","ty":"string","required":true,"description":"truck id"}]},
         "response_body":{"content_type":"application/json",
                          "fields":[{"name":"id","ty":"integer","required":true,"description":"row id"}]},
         "statuses":[{"code":201,"description":"created"}],
         "entity_refs":["delivery"],"component_refs":["api"]}
      ]
    }"#
}

fn ui_designer_response() -> &'static str {
    r#"{
      "ui":{
        "framework":"React + Vite","state_management":"Zustand","design_notes":"minimal",
        "screens":[{"id":"s_entry","name":"Entry","route":"/entry",
                    "purpose":"log a delivery",
                    "user_flow":["open","fill form","submit"],
                    "api_endpoint_refs":["e_create"],"entity_refs":["delivery"]}],
        "shared_components":[]
      },
      "business_logic":{
        "overview":"Operator-driven manual entry",
        "rules":[{"id":"r1","name":"truck required","description":"truck must not be empty",
                  "severity":"block","entity_refs":["delivery"],"triggered_by":["e_create"]}],
        "workflows":[{"id":"w1","name":"record delivery","trigger":"end of route",
                      "steps":[{"order":1,"actor":"operator","action":"open entry screen","system_response":"render form"}],
                      "rule_refs":["r1"]}]
      }
    }"#
}

fn tech_lead_response() -> &'static str {
    r#"{"tasks":[
      {"id":"t1","order":1,"goal":"scaffold Tauri app","component_refs":["ui"],
       "files_touched":["src-tauri/Cargo.toml"],"acceptance_criteria":["builds"],
       "complexity":"s"},
      {"id":"t2","order":2,"goal":"create SQLite schema","component_refs":["db"],
       "files_touched":["src-tauri/src/db.rs"],"acceptance_criteria":["migration runs"],
       "complexity":"m","depends_on":["t1"]},
      {"id":"t3","order":3,"goal":"build delivery form","component_refs":["ui","db"],
       "files_touched":["src/App.tsx"],"acceptance_criteria":["row inserted"],
       "complexity":"m","depends_on":["t2"]}
    ]}"#
}

fn estimator_response() -> &'static str {
    r#"{"oversized_task_ids":[],"notes":"all tasks fit one turn"}"#
}

fn risk_reviewer_response() -> &'static str {
    r#"{"risks":[
      {"id":"r1","description":"no backup","severity":"medium","mitigation":"daily sqlite dump"}
    ]}"#
}

fn validator_response() -> &'static str {
    r#"{"threshold":0.8,"sections":[
      {"section":"problem","score":0.9,"citation_ids":[]},
      {"section":"architecture","score":0.85,"citation_ids":["cit1","cit2"]},
      {"section":"data_model","score":0.85,"citation_ids":[]},
      {"section":"stack","score":0.9,"citation_ids":["cit1","cit2"]},
      {"section":"tasks","score":0.85,"citation_ids":[]},
      {"section":"risks","score":0.8,"citation_ids":[]}
    ]}"#
}

fn provider() -> Arc<dyn LlmProvider> {
    Arc::new(
        MockProvider::new("{}")
            .when_contains("Business request", pm_response())
            .when_contains("Problem:\n", researcher_response())
            .when_contains("Known citations", architect_response())
            .when_contains("Tasks count:", risk_reviewer_response())
            .when_contains("API endpoints:", ui_designer_response())
            .when_contains("Architecture overview:\n", tech_lead_response())
            .when_contains("Components:\n", api_designer_response())
            .when_contains("Tasks:\n", estimator_response())
            .when_contains("Plan JSON", validator_response()),
    )
}

#[tokio::test]
async fn full_pipeline_writes_valid_bundle() {
    let ctx = AgentContext::new(provider(), "mock");
    let orch = Orchestrator::new(OrchestratorConfig::default());
    let outcome = orch
        .run(&ctx, "I want to know which trucks delivered which orders today.")
        .await
        .expect("orchestrator");

    assert!(outcome.validator_passed, "validator must accept the plan");
    assert_eq!(outcome.plan.tasks.len(), 3);
    assert_eq!(outcome.plan.architecture.components.len(), 3);
    assert_eq!(outcome.plan.stack.language, "Rust");
    assert_eq!(outcome.plan.citations.len(), 2);
    assert_eq!(outcome.plan.api.endpoints.len(), 1);
    assert_eq!(outcome.plan.ui.screens.len(), 1);
    assert_eq!(outcome.plan.business_logic.rules.len(), 1);
    assert!(outcome.plan.confidence.passes());

    let tmp = std::env::temp_dir().join(format!("godsy-e2e-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp).unwrap();
    let dir = PlanBundleWriter::new(&outcome.plan).write_to(&tmp).unwrap();
    for name in [
        "PRD.md",
        "API.md",
        "UI.md",
        "CODING_AGENT_PROMPT.md",
        "risks.md",
        "tasks.json",
        "confidence.json",
        "plan.json",
        "audit.log",
    ] {
        assert!(dir.join(name).exists(), "missing {name}");
    }
    let prompt = std::fs::read_to_string(dir.join("CODING_AGENT_PROMPT.md")).unwrap();
    assert!(prompt.contains("scaffold Tauri app"));
    assert!(prompt.contains("Rust"));
    let api_md = std::fs::read_to_string(dir.join("API.md")).unwrap();
    assert!(api_md.contains("/deliveries"));
    assert!(api_md.contains("POST"));
    let ui_md = std::fs::read_to_string(dir.join("UI.md")).unwrap();
    assert!(ui_md.contains("Entry"));
    assert!(ui_md.contains("truck required"));
    std::fs::remove_dir_all(&tmp).ok();
}
