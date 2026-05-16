import { useEffect, useRef, useState } from "react";
import { Plus, CheckCircle } from "lucide-react";
import { ChatMessage, TypingIndicator } from "./ChatMessage";
import { ChatInput } from "./ChatInput";
import { useAppStore } from "@/store";
import { tauriApi, listenTauri } from "@/lib/tauri";

interface AgentProgressPayload {
  agent: string;
  status: string;
  message?: string;
}

export function ChatPanel() {
  const {
    messages,
    isPlanning,
    agents,
    addMessage,
    clearMessages,
    setPlanning,
    setAgentStatus,
    resetAgents,
    setActiveSection,
  } = useAppStore();

  const bottomRef = useRef<HTMLDivElement>(null);
  const [planReady, setPlanReady] = useState(false);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isPlanning]);

  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;

    listenTauri<AgentProgressPayload>("plan:progress", (payload) => {
      setAgentStatus(
        payload.agent,
        payload.status as "running" | "done" | "failed",
        payload.message,
      );
      if (payload.status === "done" && payload.message) {
        const label = payload.agent
          .replace(/_/g, " ")
          .replace(/\b\w/g, (c) => c.toUpperCase());
        addMessage({ role: "assistant", type: "agent_step", content: `✓ ${label} complete`, agentName: label });
      }
    }).then((fn) => { unlistenProgress = fn; });

    listenTauri<void>("plan:complete", () => {
      setPlanning(false);
      setPlanReady(true);
      addMessage({
        role: "assistant",
        type: "plan_ready",
        content:
          "**Plan complete!** Your architecture plan has been generated. Click **View Plan** to review the PRD, API design, UI architecture, task breakdown, risks, and the coding-agent prompt.",
      });
    }).then((fn) => { unlistenComplete = fn; });

    return () => { unlistenProgress?.(); unlistenComplete?.(); };
  }, [addMessage, setAgentStatus, setPlanning]);

  const handleSubmit = async (request: string) => {
    setPlanReady(false);
    resetAgents();
    addMessage({ role: "user", type: "text", content: request });
    addMessage({
      role: "assistant",
      type: "text",
      content:
        "**Understood.** Assembling your virtual engineering team now. The 9-agent pipeline will analyse your request and produce a ready-to-execute plan bundle.",
    });
    setPlanning(true);
    try {
      await tauriApi.runPlan(request);
    } catch (err) {
      setPlanning(false);
      addMessage({ role: "assistant", type: "error", content: `Pipeline error: ${String(err)}. Check your model configuration.` });
    }
  };

  const runningAgent = agents.find((a) => a.status === "running");

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-base font-semibold text-gold">Plan with Godsy</h1>
          <p className="text-xs text-text-secondary">Describe your business problem — Godsy produces the plan.</p>
        </div>
        <div className="flex items-center gap-2">
          {planReady && (
            <button
              onClick={() => setActiveSection("plans")}
              className="flex items-center gap-1.5 rounded-sm border border-gold/30 bg-gold-subtle px-3 py-1.5 text-xs font-medium text-gold transition-all hover:bg-gold hover:text-background"
            >
              <CheckCircle className="h-3.5 w-3.5" />
              View Plan
            </button>
          )}
          <button
            onClick={() => { clearMessages(); resetAgents(); setPlanReady(false); }}
            className="flex items-center gap-1.5 rounded-sm border border-border px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-gold/30 hover:text-gold"
          >
            <Plus className="h-3.5 w-3.5" />
            New Session
          </button>
        </div>
      </header>

      {/* Running agent banner */}
      {runningAgent && (
        <div className="flex items-center gap-2 border-b border-border bg-gold-subtle/40 px-6 py-2">
          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-gold" />
          <span className="text-xs text-gold-muted">{runningAgent.label} is working…</span>
        </div>
      )}

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-6 py-6">
        {messages.length === 0 ? (
          <EmptyState />
        ) : (
          <div className="mx-auto flex max-w-3xl flex-col gap-4">
            {messages.map((msg) => <ChatMessage key={msg.id} message={msg} />)}
            {isPlanning && <TypingIndicator />}
            <div ref={bottomRef} />
          </div>
        )}
      </div>

      <ChatInput onSubmit={handleSubmit} disabled={isPlanning} />
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-6 text-center">
      <div className="flex h-16 w-16 items-center justify-center rounded-sm border border-gold/20 bg-gold-subtle">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
          <path d="M16 4L4 10V22L16 28L28 22V10L16 4Z" stroke="#D4AF37" strokeWidth="1.5" strokeLinejoin="round" />
          <path d="M16 4V28M4 10L16 16L28 10" stroke="#D4AF37" strokeWidth="1.5" strokeLinecap="round" />
        </svg>
      </div>
      <div>
        <h2 className="text-lg font-semibold text-gold">Ready to plan</h2>
        <p className="mt-1 max-w-sm text-sm text-text-secondary">
          Describe any business problem. Godsy's 9-agent team will produce a complete, validated architecture plan — ready for one coding agent to ship.
        </p>
      </div>
      <div className="grid max-w-md grid-cols-2 gap-2 text-left">
        {["Track truck deliveries", "Inventory management", "Staff attendance system", "Customer order portal"].map((ex) => (
          <div key={ex} className="rounded-sm border border-border bg-surface px-3 py-2 text-xs text-text-secondary">"{ex}"</div>
        ))}
      </div>
    </div>
  );
}
