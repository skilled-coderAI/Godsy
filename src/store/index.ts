import { create } from "zustand";
import type {
  NavSection,
  ChatMessage,
  AgentStatus,
  PlanBundle,
  PlanContent,
  GodsyConfig,
  KbFile,
} from "@/types";
import { AGENT_DEFINITIONS } from "@/types";
import { generateId } from "@/lib/utils";

function initialAgents(): AgentStatus[] {
  return AGENT_DEFINITIONS.map((a) => ({
    key: a.key,
    label: a.label,
    status: "idle",
  }));
}

interface AppState {
  // Navigation
  activeSection: NavSection;
  sidebarExpanded: boolean;
  setActiveSection: (s: NavSection) => void;
  toggleSidebar: () => void;

  // Chat
  messages: ChatMessage[];
  isPlanning: boolean;
  addMessage: (msg: Omit<ChatMessage, "id" | "timestamp">) => void;
  clearMessages: () => void;
  setPlanning: (v: boolean) => void;

  // Agents
  agents: AgentStatus[];
  setAgentStatus: (key: string, status: AgentStatus["status"], summary?: string) => void;
  resetAgents: () => void;

  // Plans
  plans: PlanBundle[];
  selectedPlan: PlanBundle | null;
  planContent: PlanContent | null;
  setPlans: (plans: PlanBundle[]) => void;
  selectPlan: (plan: PlanBundle | null) => void;
  setPlanContent: (content: PlanContent | null) => void;

  // Config
  config: GodsyConfig | null;
  setConfig: (config: GodsyConfig) => void;

  // KB
  kbFiles: KbFile[];
  setKbFiles: (files: KbFile[]) => void;
  removeKbFile: (id: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Navigation
  activeSection: "chat",
  sidebarExpanded: true,
  setActiveSection: (activeSection) => set({ activeSection }),
  toggleSidebar: () =>
    set((s) => ({ sidebarExpanded: !s.sidebarExpanded })),

  // Chat
  messages: [],
  isPlanning: false,
  addMessage: (msg) =>
    set((s) => ({
      messages: [
        ...s.messages,
        { ...msg, id: generateId(), timestamp: new Date() },
      ],
    })),
  clearMessages: () => set({ messages: [] }),
  setPlanning: (isPlanning) => set({ isPlanning }),

  // Agents
  agents: initialAgents(),
  setAgentStatus: (key, status, summary) =>
    set((s) => ({
      agents: s.agents.map((a) =>
        a.key === key
          ? {
              ...a,
              status,
              summary,
              startedAt: status === "running" ? Date.now() : a.startedAt,
              durationMs:
                status === "done" || status === "failed"
                  ? a.startedAt
                    ? Date.now() - a.startedAt
                    : undefined
                  : undefined,
            }
          : a,
      ),
    })),
  resetAgents: () => set({ agents: initialAgents() }),

  // Plans
  plans: [],
  selectedPlan: null,
  planContent: null,
  setPlans: (plans) => set({ plans }),
  selectPlan: (selectedPlan) => set({ selectedPlan, planContent: null }),
  setPlanContent: (planContent) => set({ planContent }),

  // Config
  config: null,
  setConfig: (config) => set({ config }),

  // KB
  kbFiles: [],
  setKbFiles: (kbFiles) => set({ kbFiles }),
  removeKbFile: (id) =>
    set((s) => ({ kbFiles: s.kbFiles.filter((f) => f.id !== id) })),
}));
