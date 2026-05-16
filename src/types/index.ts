export type NavSection = "chat" | "plans" | "monitor" | "kb" | "settings" | "history";

export type AgentStatusKind = "idle" | "running" | "done" | "failed";

export interface AgentStatus {
  key: string;
  label: string;
  status: AgentStatusKind;
  durationMs?: number;
  summary?: string;
  startedAt?: number;
}

export type MessageRole = "user" | "assistant";
export type MessageType = "text" | "plan_ready" | "agent_step" | "error";

export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  agentName?: string;
  timestamp: Date;
  type: MessageType;
}

export interface PlanBundle {
  id: string;
  title: string;
  createdAt: string;
  status: "complete" | "failed" | "running";
  outDir: string;
}

export interface PlanContent {
  prd?: string;
  api?: string;
  ui?: string;
  tasks?: string;
  risks?: string;
  prompt?: string;
}

export interface GodsyConfig {
  provider: "ollama" | "ollama_cloud" | "cloudflare_workers";
  model: string;
  modelUrl: string;
  apiKey: string;
  grounding: "none" | "vane";
  groundingUrl: string;
  outDir: string;
  confidenceThreshold: number;
}

export interface KbFile {
  id: string;
  name: string;
  size: number;
  fileType: string;
  addedAt: string;
}

export const AGENT_DEFINITIONS: { key: string; label: string }[] = [
  { key: "product_manager", label: "Product Manager" },
  { key: "researcher", label: "Researcher" },
  { key: "architect", label: "Architect" },
  { key: "api_designer", label: "API Designer" },
  { key: "ui_designer", label: "UI Designer" },
  { key: "tech_lead", label: "Tech Lead" },
  { key: "estimator", label: "Estimator" },
  { key: "risk_reviewer", label: "Risk Reviewer" },
  { key: "validator", label: "Validator" },
];
