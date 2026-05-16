# Godsy — Development Progress Tracker

Last updated: 2026-05-16 (Vane-only + Ollama Cloud + Knowledge Base foundation)

## Status Legend

- [ ] not started
- [~] in progress
- [x] done
- [!] blocked / needs decision

## Strategy

Build the planning engine as a Cargo workspace of pure-Rust crates first, exercised by a small CLI. Attach the Tauri desktop shell only after the engine produces a valid Plan Bundle end-to-end. This keeps `.\prd.md` §9 (agent team) and §12 (plan bundle) as the unit of value, not the UI.

## Phase 0 — Bootstrap

- [x] Rebrand PRD and AGENTS.md to Godsy
- [x] Create `.\PROGRESS.md` (this file)
- [x] Add `.\.gitignore`
- [x] Initialize Cargo workspace (`.\Cargo.toml`)
- [x] Add workspace `rustfmt.toml` and `clippy` lints
- [x] `git init` and first commit

## Phase 1 — Core Domain (`crates/godsy-core`)

Pure types and serialization for everything that flows between agents and onto disk.

- [x] Crate scaffold
- [x] `Plan` struct (problem, architecture, data model, tasks, risks, confidence)
- [x] `Task` struct (id, goal, inputs, outputs, files, acceptance, complexity)
- [x] `Citation` struct (source kind, locator, snippet, retrieved_at)
- [x] `ConfidenceReport` struct (per-section score + threshold logic)
- [x] `PlanBundle` writer to disk per `.\prd.md` §12
- [x] Unit tests: round-trip serde, bundle layout
- [x] Structural-verification helper (Layer 3 of `.\prd.md` §10)

## Phase 2 — LLM Provider (`crates/godsy-llm`)

- [x] `LlmProvider` trait (chat + structured-JSON calls)
- [x] Ollama HTTP client (default `http://localhost:11434`)
- [x] **Ollama Cloud** support (bearer-auth API key against `https://ollama.com` or any auth-proxied Ollama deployment; same `OllamaProvider` via `with_api_key`)
- [ ] Cloudflare Workers AI client (opt-in, deferred)
- [x] Provider config + routing struct
- [x] Mock provider for tests
- [x] Unit tests against mock

## Phase 3 — Agent Framework (`crates/godsy-agents`)

- [x] `PlanningAgent` trait (input/output typed messages)
- [x] Session / message-bus primitive (in-process channels for MVP)
- [x] Prompt-template loader
- [~] swarms-rs integration (deferred to Phase 5; in-process orchestrator first)

## Phase 4 — Planning Agents

One module per role, each with prompt template + unit test against MockLlm.

- [x] Product Manager
- [x] Researcher (stub — grounding tools land in Phase 6)
- [x] Architect
- [x] API Designer (produces `API.md`)
- [x] UI Designer (produces `UI.md`, including business-logic analysis)
- [x] Tech Lead
- [x] Estimator
- [x] Risk Reviewer
- [x] Validator (citation + structural checks)

## Phase 5 — Orchestrator

- [x] In-process sequential orchestrator running the 9-agent pipeline
- [x] Validator rejection loop (route back to Architect on low confidence)
- [x] End-to-end integration test (`crates/godsy-agents/tests/end_to_end.rs`) — runs all 9 agents against `MockProvider`, asserts validator passes, plan bundle written with `PRD.md`, `API.md`, `UI.md`, `CODING_AGENT_PROMPT.md`, etc.
- [ ] swarms-rs-backed orchestrator (parity test)

## Phase 6 — Grounding

- [x] `GroundingProvider` trait (gateway-agnostic; consumed by `ResearcherAgent`)
- [x] **Vane** client (`POST /api/search`; parses `{message, sources[]}` into `Citation { kind: Web }`) — sole web-grounding gateway
- [x] `[grounding]` block in `godsy.toml` selecting `none` | `vane` + `base_url` + Vane chat/embedding settings
- [x] Citation resolver (mechanical Layer 1 enforcement) — `verify_citations` in `godsy-core`
- [-] SearXNG direct client — **dropped**: Vane already wraps SearXNG, no value running both
- [~] Knowledge-base ingestion (PDF/DOCX/XLSX → chunks)
- [~] Local vector store (`sqlite-vec`)

## Phase 7 — CLI (`crates/godsy-cli`)

- [x] `godsy init` — write default `godsy.toml`
- [x] `godsy plan "<request>"` — loads `godsy.toml`, runs full pipeline, writes bundle
- [x] CLI overrides (`--config`, `--out`, `--model`, `--model-url`/`--ollama-url`, `--provider`, `--api-key`, `--grounding`, `--grounding-url`, `--strict`)
- [x] Env-var overrides (`GODSY_MODEL`, `GODSY_MODEL_BASE_URL`, `GODSY_OUT_DIR`, `GODSY_GROUNDING_URL`, `OLLAMA_API_KEY`, `GODSY_API_KEY`)
- [x] `--explain` flag to dump per-agent traces (writes `<bundle>/explain.jsonl`)

## Phase 8 — Tauri Shell (`src-tauri/` + `src/`)

- [ ] Scaffold Tauri app
- [ ] IPC commands wrapping `godsy-agents`
- [ ] Chat UI
- [ ] Plan Viewer
- [ ] Agent Monitor
- [ ] Knowledge Base UI
- [ ] Model Configuration

## Phase 9 — Verification & Hardening

- [x] Citation-resolution unit tests (`verify_citations` — 3 tests in `godsy-core::verify`)
- [x] Structural-verification integration test (`godsy_core::verify` — 3 unit tests)
- [x] End-to-end test: stub LLM → valid Plan Bundle
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] `cargo fmt --all -- --check` clean

## Open Decisions

- **Vector store**: `sqlite-vec` (single-file) vs embedded `qdrant`. Leaning `sqlite-vec`.
- **Frontend framework**: React + Vite vs SvelteKit. Defer until Phase 8.
- **swarms-rs adoption**: Use directly or wrap in our own `Orchestrator`? Plan: wrap, so MVP can ship without it.

## Session Log

- **2026-05-15**: Tracker created. Phases 0–5 + 7 + Phase 9 partial landed in a single session:
  - Cargo workspace with 4 crates: `godsy-core`, `godsy-llm`, `godsy-agents`, `godsy-cli`.
  - All 7 planning agents implemented against a typed `LlmProvider` trait.
  - `MockProvider` enables deterministic testing without a real LLM.
  - `OllamaProvider` ready for the real path (`POST /api/chat`, `format=json`).
  - Plan Bundle writer emits all 9 files defined in `prd.md` §12.
  - Mechanical structural verifier rejects dangling refs and forward dependencies.
  - **15 tests passing** (4 core, 2 llm, 8 agent unit, 1 end-to-end). Clippy clean with `-D warnings`.
  - `godsy plan "<request>"` CLI binary builds; needs a running Ollama on `:11434` for the live path.

- **2026-05-15 (hardening pass)**: warnings → errors, mocks gated, real config, output restructure:
  - Workspace `[workspace.lints]` denies `unused`, `nonstandard_style`, `future_incompatible`, `rust_2018_idioms`, `unreachable_pub`, `missing_debug_implementations`. Clippy `all`/`correctness`/`suspicious`/`perf` denied; `pedantic` warn with a curated allow-list for stylistic-only lints. Real `#[derive(Debug)]`, `#[must_use]`, and code shape fixes applied — no per-instance `#[allow(...)]`.
  - `MockProvider` moved behind `#[cfg(any(test, feature = "test-support"))]`; downstream crates enable `test-support` only as a dev-dependency. `OllamaProvider` is the only LLM path compiled into the release binary.
  - Real `GodsyConfig` (`crates/godsy-core/src/config.rs`): TOML, `deny_unknown_fields`, range-validated temperature & confidence threshold, env-var overrides. CLI loads (or writes-then-loads) `godsy.toml` instead of relying on hardcoded defaults.
  - Plan output reshaped around three primary markdown documents the user asked for: **`PRD.md`** (the execution plan), **`API.md`** (backend API documentation), **`UI.md`** (UI architecture + business-logic analysis), plus `CODING_AGENT_PROMPT.md` as the single-prompt handoff. Machine-readable companions: `plan.json`, `tasks.json`, `confidence.json`, `risks.md`, `sources/*.json`, `audit.log`.
  - Two new planning agents — `ApiDesignerAgent` and `UiDesignerAgent` — added between Architect and Tech Lead. Orchestrator now runs **9 agents** in sequence with a Validator rejection loop.
  - `Plan` extended with `ApiSpec`, `UiSpec`, and `BusinessLogicAnalysis`; structural verifier and bundle writer updated; integration test rewritten for the 9-agent pipeline.
  - **20 tests passing** (7 core, 2 llm, 10 agent unit, 1 end-to-end). `cargo clippy --workspace --all-targets -- -D warnings` clean.

- **2026-05-16 (Phase 6 + --explain landed)**: 5th crate, dual-gateway grounding wired end-to-end:
  - New crate `crates/godsy-grounding` (added to workspace): `GroundingProvider` trait, `GroundingQuery`, `GroundingHit`, `GroundingError`. Implementations: `NoopGrounder` (default, offline), `SearxngGrounder` (`GET /search?format=json`), `PerplexicaGrounder` (`POST /api/search` with full Perplexica request shape — `chatModel`, `embeddingModel`, `optimizationMode`, `focusMode`). `MockGrounder` gated behind `test-support` feature.
  - Pure response parsers (`parse_searxng_response`, `parse_perplexica_response`) covered by 4 unit tests.
  - `GodsyConfig.grounding`: `GroundingKind` (`none|searxng|perplexica`), `base_url`, `max_hits`, `request_timeout_secs`, optional `[grounding.perplexica]` block (focus_mode, optimization_mode, chat_provider/model, embedding_provider/model). Range-validated; `base_url` required when provider != none. New env override `GODSY_GROUNDING_URL`.
  - `AgentContext` now carries `grounder: Arc<dyn GroundingProvider>` (defaults to `NoopGrounder`) and optional `Arc<ExplainRecorder>`. Every `chat_json` call records an `AgentTraceEvent` (agent name, timestamp, system+user prompt, response, model) when explain is enabled.
  - `ResearcherAgent` rewritten to call `ctx.grounder.search` first, seed each hit as `Citation { kind: Web, id = "g-{uuid}" }`, inject a "Grounded web hits" block into the user prompt, then call the LLM. Empty-url LLM findings are dropped. Three new researcher tests cover the no-grounding, mock-grounder-merge, and empty-url-drop paths.
  - New mechanical Layer-1 verifier `godsy_core::verify::verify_citations`: rejects empty/duplicate citation ids, empty Web urls, empty KB chunk_ids, empty ProjectFile paths, dangling stack/confidence references. `ValidatorAgent` now runs both `verify_structure` and `verify_citations` before the LLM confidence pass.
  - CLI: `--grounding {none|searxng|perplexica}`, `--grounding-url`, `--explain` (writes `<bundle>/explain.jsonl`), `--strict` (exit non-zero if validator did not pass). CLI re-validates the effective config after applying overrides via a temp-file round-trip so invalid overrides fail loudly.
  - **30 tests passing** (10 core, 2 llm, 5 grounding, 12 agent unit, 1 end-to-end). `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --all -- --check` clean (now an enforced gate).

- **2026-05-16 (Vane consolidation + Ollama Cloud)**: gateway simplification + hosted-model auth:
  - Dropped direct SearXNG path entirely. Vane (the rebrand of Perplexica, same HTTP protocol) is now the only web-grounding gateway. Reasoning: Vane already wraps SearXNG with re-ranking + answer synthesis; running both was duplicate config surface with no extra signal. `crates/godsy-grounding/src/searxng.rs` deleted; `perplexica.rs` renamed to `vane.rs`; types renamed `PerplexicaGrounder` → `VaneGrounder`, `PerplexicaSettings` → `VaneSettings`, parser `parse_perplexica_response` → `parse_vane_response`. `GroundingKind` reduced to `None | Vane`. `GroundingConfig.perplexica` → `GroundingConfig.vane`.
  - **Ollama Cloud** added as a real provider option. New `ProviderKind::OllamaCloud`. `ModelConfig.api_key: String` field (default empty, `serde(default)`). `OllamaProvider::with_api_key(base_url, api_key, timeout)` attaches `Authorization: Bearer <key>` when the key is non-empty — works against `https://ollama.com` and any auth-proxied Ollama. Empty keys = no header, so the same constructor serves both local and cloud paths.
  - Validation now rejects `provider = "ollama_cloud"` with empty `api_key`, pointing the user at `OLLAMA_API_KEY` / `GODSY_API_KEY` env vars or the TOML field. New unit test `rejects_ollama_cloud_without_api_key`.
  - CLI: `--grounding {none|vane}` (no more `searxng|perplexica`), `--provider {ollama|ollama_cloud|cloudflare_workers}`, `--api-key`. Env: `OLLAMA_API_KEY`, `GODSY_API_KEY` (the latter wins).
  - Refactored `Cmd` to box `PlanArgs` (large_enum_variant clippy fix — real fix, no `#[allow]`).
  - **29 tests passing** (11 core, 2 llm, 3 grounding, 12 agent unit, 1 end-to-end). `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --all -- --check` clean.

- **2026-05-16**: Architecture doc + dual-gateway grounding plan:
  - Added `.\ARCHITECTURE.md` — system context, code topology, dependency direction across the 4 crates, 9-agent pipeline rationale, plan-bundle anatomy, 4-layer hallucination defence, configuration story, threat model, quality gates, extension seams, and the deliberate "what this architecture refuses" list.
  - `.\prd.md` §8 Grounding rewritten to support **two gateways**: SearXNG direct and **Perplexica / Vane** ([ItzCrazyKns/Perplexica](https://github.com/ItzCrazyKns/Perplexica)). Both feed the same `Citation { kind: Web }` so the rest of the pipeline is gateway-agnostic.
  - Phase 6 tracker updated with `Grounder` trait + Perplexica client + `[grounding]` config block tasks.

- **2026-05-16 (Knowledge Base foundation)**:
  - Added `crates/godsy-kb` crate to workspace
  - Implemented knowledge base foundation with: chunker, error handling, extractor, grounding integration, ingestion pipeline, storage abstraction
  - Knowledge base designed for future vector store integration (`sqlite-vec` or embedded `qdrant`)
  - GroundingProvider trait extended to support knowledge base citations

## How to Run

```cmd
cargo test --manifest-path c:\Users\Jay\git\Godsy\Cargo.toml --workspace
cargo clippy --manifest-path c:\Users\Jay\git\Godsy\Cargo.toml --workspace --all-targets -- -D warnings
cargo run --manifest-path c:\Users\Jay\git\Godsy\Cargo.toml -p godsy-cli -- init --config godsy.toml
cargo run --manifest-path c:\Users\Jay\git\Godsy\Cargo.toml -p godsy-cli -- plan "I want to track which trucks delivered orders today" --config godsy.toml
```

The CLI invocation requires Ollama running locally with the configured model pulled (default `qwen2.5`; override via `godsy.toml`, `--model`, or `GODSY_MODEL`). The generated bundle directory contains `PRD.md`, `API.md`, `UI.md`, and `CODING_AGENT_PROMPT.md` — paste `CODING_AGENT_PROMPT.md` into one coding agent to ship the planned project in a single run.
