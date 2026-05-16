# Repository Guidelines

## Project Status

This repository is in the **planning / pre-implementation phase**. The only artifact present is `.\prd.md`, the Product Requirements Document for **Godsy** — a local-first, multi-agent **architecture and planning studio** for non-IT businesses. Godsy itself does **not** generate code; it produces validated implementation plans for a single downstream coding agent (Claude Code, Cursor, Zencoder, Aider, etc.) to execute. No source, build system, tests, or git history exist yet. Contributors adding the first code must also add the corresponding tooling (manifest, lint config, CI) and replace the placeholders in this file with real commands.

## Product Intent (read before coding)

- Godsy is a **planning tool, not a build tool.** No agent in this codebase should generate, compile, or run user-application code.
- The final output of every session is a **Plan Bundle** on disk (see `.\prd.md` §12), whose centerpiece is `07_coding_agent_prompt.md` — a single prompt designed to drive one external coding agent end-to-end.
- All agent roles are **planning roles**: Product Manager, Researcher, Architect, Tech Lead, Estimator, Risk Reviewer, Validator (`.\prd.md` §9). Do not introduce a "Coder" or "Test Engineer" agent — that is intentionally out of scope.

## Project Structure & Module Organization

Current layout:

- `.\prd.md` — full Godsy product spec (vision, agent roles, plan bundle format, roadmap).
- `.\AGENTS.md` — this file.

Planned layout (per `.\prd.md` §7–§14) when implementation begins:

- Desktop shell: **Tauri** app — Rust backend under `.\src-tauri\`, web frontend under `.\src\`.
- Agent orchestration: **`swarms-rs`** inside `.\src-tauri\src\agents\`, one module per planning role (`product_manager.rs`, `researcher.rs`, `architect.rs`, `tech_lead.rs`, `estimator.rs`, `risk_reviewer.rs`, `validator.rs`).
- Model routing: local via **Ollama** (`qwen2.5`, `deepseek-r1`, `llama3.1`); opt-in cloud via **Cloudflare Workers AI**. Used for **planning text and structured JSON only** — never to emit shippable user code.
- Grounding (operator picks one gateway, both feed the same `Citation` type): **SearXNG direct** *or* **Perplexica / Vane** ([ItzCrazyKns/Perplexica](https://github.com/ItzCrazyKns/Perplexica)) at `http://localhost:3000` for AI-answered, source-cited search. Plus a local vector store (`sqlite-vec` or embedded `qdrant`) for knowledge-base citations.
- Storage: SQLite for sessions, filesystem for plan bundles, JSONL for audit logs, OS keychain for credentials (`.\prd.md` §14).
- UI modules: Chat, Plan Viewer, Agent Monitor, Knowledge Base, Model Configuration, Plan History (`.\prd.md` §13).

## Build, Test, and Development Commands

No build system is configured yet. Once Tauri is scaffolded (`cargo create-tauri-app`), the expected commands will be:

- `npm install` / `pnpm install` — install frontend deps.
- `npm run tauri dev` — run Godsy in dev mode.
- `npm run tauri build` — produce release binaries.
- `cargo test --manifest-path src-tauri/Cargo.toml` — run Rust tests; append `-- <test_name>` to target one.
- `cargo fmt --manifest-path src-tauri/Cargo.toml` and `cargo clippy --all-targets -- -D warnings` — format and lint Rust.

Replace this section with the actual scripts the moment `package.json` and `Cargo.toml` land.

## Coding Style & Naming Conventions

No linter/formatter configs exist yet. Adopt these defaults when scaffolding:

- **Rust**: `rustfmt` defaults (4-space indent); `clippy` with `-D warnings`. Files/modules `snake_case`, types `CamelCase`, constants `SCREAMING_SNAKE_CASE`.
- **Frontend (TS/JS)**: Prettier + ESLint; 2-space indent; components `PascalCase`, hooks `useCamelCase`.
- One file per agent role, named after the role (`architect.rs`, `validator.rs`) to match `.\prd.md` §9.
- Plan-bundle file names follow the fixed `NN_section_name.md` pattern in `.\prd.md` §12 — do not rename without updating the PRD.

## Testing Guidelines

No test suite exists. Per `.\prd.md` §18, key targets are >80% of plans executable end-to-end by a single coding agent and <3% hallucinated-citation rate. Test additions should reflect that:

- Unit tests via `cargo test` colocated as `#[cfg(test)]` modules.
- Integration tests under `.\src-tauri\tests\` exercising end-to-end planning flows.
- A **citation-resolution test** that mechanically verifies every citation in a generated plan resolves (Layer 1 of `.\prd.md` §10).
- A **structural-verification test** that parses `04_tasks.json` and confirms cross-references to the data model and architecture (Layer 3 of `.\prd.md` §10).

## Commit & Pull Request Guidelines

No git history is available to derive conventions. Use **Conventional Commits** (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`) with subjects under 72 characters. PRs should link the relevant `.\prd.md` section, describe scope, list verification steps, and include screenshots for UI changes. Any change that affects the plan-bundle schema must update `.\prd.md` §12 in the same PR.

## Security & Configuration

Per `.\prd.md` §15, Godsy is local-first: no telemetry, no network calls without explicit user opt-in. Store API keys (Cloudflare, SearXNG) in the OS keychain; never log them. Scope agent filesystem access to the active workspace. Never commit `.env` files, model credentials, or sample plan bundles containing real business data.
