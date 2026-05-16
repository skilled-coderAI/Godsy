/**
 * Thin wrapper around @tauri-apps/api invoke with a mock fallback
 * for browser-only development.
 */

type InvokeArgs = Record<string, unknown>;

async function invokeCommand<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  // In a Tauri context
  if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }
  // Browser mock fallback
  return mockInvoke<T>(cmd, args);
}

// ─── Mock implementations for browser dev ────────────────────────────────────

import type { GodsyConfig, PlanBundle, PlanContent, KbFile } from "@/types";
import { MOCK_CONFIG, MOCK_PLANS, MOCK_KB_FILES } from "@/lib/mockData";

function mockInvoke<T>(cmd: string, _args?: InvokeArgs): Promise<T> {
  const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

  switch (cmd) {
    case "get_config":
      return delay(100).then(() => MOCK_CONFIG as unknown as T);
    case "save_config":
      return delay(200).then(() => undefined as unknown as T);
    case "list_plans":
      return delay(300).then(() => MOCK_PLANS as unknown as T);
    case "get_plan":
      return delay(200).then(
        () =>
          ({
            prd: "# Project Requirements\n\nThis is a mock PRD document.\n\n## Overview\n\nYour planning document will appear here after running the agents.",
            api: "# API Design\n\nBackend endpoints and data contracts.",
            ui: "# UI Architecture\n\nComponent breakdown and wireframes.",
            tasks: JSON.stringify([{ id: "t1", goal: "Setup project" }], null, 2),
            risks: "# Risk Analysis\n\nIdentified risks and mitigations.",
            prompt: "# Coding Agent Prompt\n\nPaste this into your coding agent to ship the project.",
          } as unknown as T),
      );
    case "list_kb_files":
      return delay(200).then(() => MOCK_KB_FILES as unknown as T);
    case "delete_kb_file":
      return delay(100).then(() => undefined as unknown as T);
    case "upload_kb_file":
      return delay(300).then(() => undefined as unknown as T);
    case "run_plan":
      return delay(100).then(() => "Mock pipeline started" as unknown as T);
    default:
      return Promise.reject(new Error(`Unknown command: ${cmd}`));
  }
}

// ─── Public API ──────────────────────────────────────────────────────────────

export const tauriApi = {
  getConfig: () => invokeCommand<GodsyConfig>("get_config"),
  saveConfig: (config: GodsyConfig) =>
    invokeCommand<void>("save_config", { config }),
  listPlans: () => invokeCommand<PlanBundle[]>("list_plans"),
  getPlan: (outDir: string) =>
    invokeCommand<PlanContent>("get_plan", { outDir }),
  runPlan: (request: string) =>
    invokeCommand<string>("run_plan", { request }),
  listKbFiles: () => invokeCommand<KbFile[]>("list_kb_files"),
  deleteKbFile: (id: string) =>
    invokeCommand<void>("delete_kb_file", { id }),
  uploadKbFile: (path: string) =>
    invokeCommand<void>("upload_kb_file", { path }),
};

/**
 * Opens a file picker dialog (Tauri) and uploads selected files to the KB.
 * No-op in browser dev mode.
 */
export async function openFileDialogAndUpload(): Promise<void> {
  if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const result = await open({
      multiple: true,
      filters: [
        {
          name: "Planning Documents",
          extensions: ["pdf", "docx", "xlsx", "md", "csv"],
        },
      ],
    });
    if (!result) return;
    const paths = Array.isArray(result) ? result : [result];
    for (const path of paths) {
      await tauriApi.uploadKbFile(path);
    }
  }
  // In browser: caller falls back to HTML file-input
}

export async function listenTauri<T>(
  event: string,
  callback: (payload: T) => void,
): Promise<() => void> {
  if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<T>(event, (e) => callback(e.payload));
    return unlisten;
  }
  // No-op in browser
  return () => {};
}
