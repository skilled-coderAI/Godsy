import {
  User, Search, Building2, Code2, Palette,
  Wrench, Calculator, Shield, CheckCircle2,
  Clock, Loader2, XCircle, Minus,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import { formatDuration } from "@/lib/utils";
import type { AgentStatus, AgentStatusKind } from "@/types";

const AGENT_ICONS: Record<string, React.ElementType> = {
  product_manager: User,
  researcher: Search,
  architect: Building2,
  api_designer: Code2,
  ui_designer: Palette,
  tech_lead: Wrench,
  estimator: Calculator,
  risk_reviewer: Shield,
  validator: CheckCircle2,
};

const AGENT_DESCRIPTIONS: Record<string, string> = {
  product_manager: "Clarifies real business need and defines success criteria",
  researcher: "Grounds decisions with verified sources and citations",
  architect: "Designs layered system architecture",
  api_designer: "Produces typed API contracts and endpoint specs",
  ui_designer: "Maps UI components and business-logic flows",
  tech_lead: "Selects stack, defines dev environment and tooling",
  estimator: "Breaks work into ordered atomic tasks",
  risk_reviewer: "Identifies risks, unknowns, and integration gaps",
  validator: "Verifies citations, structure, and confidence scores",
};

export function AgentMonitor() {
  const { agents, isPlanning } = useAppStore();
  const doneCount = agents.filter((a) => a.status === "done").length;

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-base font-semibold text-gold">Agent Monitor</h1>
          <p className="text-xs text-text-secondary">9-agent planning pipeline status</p>
        </div>
        {isPlanning && (
          <div className="flex items-center gap-2">
            <Loader2 className="h-3.5 w-3.5 animate-spin text-gold" />
            <span className="text-xs text-gold">{doneCount} / 9 complete</span>
          </div>
        )}
        {!isPlanning && doneCount > 0 && (
          <span className="text-xs text-text-secondary">{doneCount} / 9 completed last run</span>
        )}
      </header>

      {/* Pipeline progress bar */}
      {(isPlanning || doneCount > 0) && (
        <div className="border-b border-border bg-surface px-6 py-3">
          <div className="flex items-center gap-3">
            <div className="h-1 flex-1 overflow-hidden rounded-full bg-surface-elevated">
              <div
                className="h-full rounded-full bg-gold transition-all duration-500"
                style={{ width: `${(doneCount / 9) * 100}%` }}
              />
            </div>
            <span className="text-[10px] tabular-nums text-text-secondary">
              {Math.round((doneCount / 9) * 100)}%
            </span>
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-6">
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {agents.map((agent, i) => (
            <AgentCard key={agent.key} agent={agent} index={i} />
          ))}
        </div>
      </div>
    </div>
  );
}

function AgentCard({ agent, index }: { agent: AgentStatus; index: number }) {
  const Icon = AGENT_ICONS[agent.key] ?? User;
  const description = AGENT_DESCRIPTIONS[agent.key] ?? "";

  return (
    <div
      className={cn(
        "rounded-sm border p-4 transition-all duration-300",
        agent.status === "running"
          ? "border-gold/40 bg-gold-subtle shadow-[0_0_16px_rgba(212,175,55,0.1)]"
          : agent.status === "done"
            ? "border-border bg-surface"
            : agent.status === "failed"
              ? "border-destructive/30 bg-destructive/5"
              : "border-border/50 bg-surface/50",
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-start gap-3">
          <div
            className={cn(
              "flex h-8 w-8 shrink-0 items-center justify-center rounded-sm",
              agent.status === "running"
                ? "bg-gold/10 ring-1 ring-gold/30"
                : agent.status === "done"
                  ? "bg-gold/5 ring-1 ring-gold/10"
                  : "bg-surface-elevated ring-1 ring-border",
            )}
          >
            <Icon
              className={cn(
                "h-4 w-4",
                agent.status === "running"
                  ? "text-gold"
                  : agent.status === "done"
                    ? "text-gold-muted"
                    : "text-text-muted",
              )}
            />
          </div>
          <div className="min-w-0">
            <p className="text-xs font-semibold uppercase tracking-wider text-gold-muted">
              {String(index + 1).padStart(2, "0")}
            </p>
            <p
              className={cn(
                "text-sm font-medium",
                agent.status === "running" ? "text-gold" : "text-text-secondary",
              )}
            >
              {agent.label}
            </p>
          </div>
        </div>
        <StatusBadge status={agent.status} />
      </div>

      <p className="mt-3 text-[11px] leading-relaxed text-text-muted">{description}</p>

      {agent.durationMs !== undefined && (
        <div className="mt-2 flex items-center gap-1 text-[10px] text-text-muted">
          <Clock className="h-2.5 w-2.5" />
          {formatDuration(agent.durationMs)}
        </div>
      )}
      {agent.status === "running" && (
        <div className="mt-2 flex items-center gap-1 text-[10px] text-gold">
          <Loader2 className="h-2.5 w-2.5 animate-spin" />
          Working…
        </div>
      )}
    </div>
  );
}

function StatusBadge({ status }: { status: AgentStatusKind }) {
  const map: Record<AgentStatusKind, { label: string; cls: string; Icon: React.ElementType }> = {
    idle: { label: "Idle", cls: "text-text-muted border-border/50", Icon: Minus },
    running: { label: "Running", cls: "text-gold border-gold/30 bg-gold/10", Icon: Loader2 },
    done: { label: "Done", cls: "text-success border-success/30 bg-success/5", Icon: CheckCircle2 },
    failed: { label: "Failed", cls: "text-destructive border-destructive/30 bg-destructive/5", Icon: XCircle },
  };
  const { label, cls, Icon } = map[status];
  return (
    <span className={cn("flex items-center gap-1 rounded-xs border px-1.5 py-0.5 text-[10px] font-medium", cls)}>
      <Icon className={cn("h-2.5 w-2.5", status === "running" && "animate-spin")} />
      {label}
    </span>
  );
}
