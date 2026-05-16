import { useEffect, useState } from "react";
import { Copy, Check, Download, FileText } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import { tauriApi } from "@/lib/tauri";
import type { PlanContent } from "@/types";

const TABS: { key: keyof PlanContent; label: string }[] = [
  { key: "prd", label: "PRD" },
  { key: "api", label: "API" },
  { key: "ui", label: "UI" },
  { key: "tasks", label: "Tasks" },
  { key: "risks", label: "Risks" },
  { key: "prompt", label: "Agent Prompt" },
];

export function PlanViewer() {
  const { plans, selectedPlan, planContent, selectPlan, setPlanContent } = useAppStore();
  const [activeTab, setActiveTab] = useState<keyof PlanContent>("prd");
  const [copied, setCopied] = useState(false);

  // Auto-select latest plan
  useEffect(() => {
    if (!selectedPlan && plans.length > 0) {
      selectPlan(plans[0]);
    }
  }, [plans, selectedPlan, selectPlan]);

  useEffect(() => {
    if (selectedPlan && !planContent) {
      tauriApi.getPlan(selectedPlan.outDir).then(setPlanContent).catch(console.error);
    }
  }, [selectedPlan, planContent, setPlanContent]);

  const content = planContent?.[activeTab];

  const handleCopy = () => {
    if (content) {
      navigator.clipboard.writeText(content).then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      });
    }
  };

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-base font-semibold text-gold">Plan Viewer</h1>
          <p className="text-xs text-text-secondary">
            {selectedPlan ? selectedPlan.title : "No plan selected"}
          </p>
        </div>
        {content && (
          <button
            onClick={handleCopy}
            className="flex items-center gap-1.5 rounded-sm border border-border px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-gold/30 hover:text-gold"
          >
            {copied ? <Check className="h-3.5 w-3.5 text-success" /> : <Copy className="h-3.5 w-3.5" />}
            {copied ? "Copied!" : "Copy"}
          </button>
        )}
      </header>

      {/* Tabs */}
      <div className="flex border-b border-border bg-surface px-6">
        {TABS.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={cn(
              "border-b-2 px-4 py-3 text-xs font-medium transition-colors",
              activeTab === tab.key
                ? "border-gold text-gold"
                : "border-transparent text-text-secondary hover:text-gold",
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-6">
        {!selectedPlan ? (
          <EmptyPlans />
        ) : !planContent ? (
          <div className="flex h-full items-center justify-center">
            <div className="flex flex-col items-center gap-3 text-text-secondary">
              <div className="h-5 w-5 animate-spin rounded-full border-2 border-gold/20 border-t-gold" />
              <span className="text-xs">Loading plan…</span>
            </div>
          </div>
        ) : !content ? (
          <div className="flex h-full items-center justify-center text-text-secondary text-xs">
            No content for this section.
          </div>
        ) : activeTab === "tasks" ? (
          <TasksView content={content} />
        ) : (
          <div className="prose-godsy mx-auto max-w-3xl">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
          </div>
        )}
      </div>
    </div>
  );
}

function TasksView({ content }: { content: string }) {
  try {
    const tasks = JSON.parse(content) as Array<{ id: string; goal: string; inputs?: string[]; outputs?: string[] }>;
    return (
      <div className="mx-auto max-w-3xl space-y-2">
        {tasks.map((t, i) => (
          <div key={t.id} className="rounded-sm border border-border bg-surface p-4">
            <div className="flex items-start gap-3">
              <span className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-xs border border-gold/20 bg-gold-subtle text-[10px] font-bold text-gold">
                {i + 1}
              </span>
              <div>
                <p className="text-sm font-medium text-gold">{t.goal}</p>
                {t.inputs && (
                  <p className="mt-1 text-[11px] text-text-secondary">Inputs: {t.inputs.join(", ")}</p>
                )}
                {t.outputs && (
                  <p className="mt-0.5 text-[11px] text-text-secondary">Outputs: {t.outputs.join(", ")}</p>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  } catch {
    return (
      <div className="prose-godsy mx-auto max-w-3xl">
        <pre className="text-xs text-gold-muted">{content}</pre>
      </div>
    );
  }
}

function EmptyPlans() {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 text-center">
      <div className="flex h-14 w-14 items-center justify-center rounded-sm border border-gold/20 bg-gold-subtle">
        <FileText className="h-6 w-6 text-gold-muted" />
      </div>
      <div>
        <p className="text-sm font-medium text-gold">No plans yet</p>
        <p className="mt-1 text-xs text-text-secondary">
          Run the chat to generate your first architecture plan.
        </p>
      </div>
      <Download className="hidden" />
    </div>
  );
}
