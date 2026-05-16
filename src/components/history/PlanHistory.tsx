import { useEffect } from "react";
import {
  Clock,
  ArrowRight,
  RefreshCw,
  CheckCircle2,
  XCircle,
  Loader2,
  FileText,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import { tauriApi } from "@/lib/tauri";
import { formatRelativeTime } from "@/lib/utils";
import type { PlanBundle } from "@/types";

export function PlanHistory() {
  const { plans, setPlans, selectPlan, setActiveSection } = useAppStore();

  const refresh = () => {
    tauriApi.listPlans().then(setPlans).catch(console.error);
  };

  useEffect(() => {
    refresh();
  }, []);

  const openPlan = (plan: PlanBundle) => {
    selectPlan(plan);
    setActiveSection("plans");
  };

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-base font-semibold text-gold">Plan History</h1>
          <p className="text-xs text-text-secondary">
            All generated architecture plans.
          </p>
        </div>
        <button
          onClick={refresh}
          className="flex items-center gap-1.5 rounded-sm border border-border px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-gold/30 hover:text-gold"
        >
          <RefreshCw className="h-3.5 w-3.5" />
          Refresh
        </button>
      </header>

      {/* Stats strip */}
      {plans.length > 0 && (
        <div className="border-b border-border bg-surface px-6 py-3">
          <div className="flex items-center gap-6 text-[11px]">
            <Stat
              label="Total"
              value={plans.length}
              color="text-text-secondary"
            />
            <Stat
              label="Complete"
              value={plans.filter((p) => p.status === "complete").length}
              color="text-success"
            />
            <Stat
              label="Failed"
              value={plans.filter((p) => p.status === "failed").length}
              color="text-destructive"
            />
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto px-6 py-6">
        {plans.length === 0 ? (
          <EmptyHistory />
        ) : (
          <div className="mx-auto max-w-3xl space-y-2">
            {plans.map((plan) => (
              <PlanCard
                key={plan.id}
                plan={plan}
                onOpen={() => openPlan(plan)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function Stat({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: string;
}) {
  return (
    <div className="flex items-center gap-1.5">
      <span className={cn("text-sm font-semibold tabular-nums", color)}>
        {value}
      </span>
      <span className="text-text-muted">{label}</span>
    </div>
  );
}

function PlanCard({
  plan,
  onOpen,
}: {
  plan: PlanBundle;
  onOpen: () => void;
}) {
  const isComplete = plan.status === "complete";
  const isFailed = plan.status === "failed";
  const isRunning = plan.status === "running";

  const StatusIcon = isComplete ? CheckCircle2 : isFailed ? XCircle : Loader2;

  return (
    <div className="group flex items-center gap-4 rounded-sm border border-border bg-surface px-5 py-4 transition-all hover:border-gold/25 hover:bg-surface-elevated">
      {/* Status icon */}
      <div
        className={cn(
          "flex h-10 w-10 shrink-0 items-center justify-center rounded-sm ring-1",
          isComplete
            ? "bg-success/5 ring-success/20"
            : isFailed
              ? "bg-destructive/5 ring-destructive/20"
              : "bg-gold-subtle ring-gold/20",
        )}
      >
        <StatusIcon
          className={cn(
            "h-4 w-4",
            isComplete
              ? "text-success"
              : isFailed
                ? "text-destructive"
                : "animate-spin text-gold",
          )}
        />
      </div>

      {/* Plan info */}
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-gold">{plan.title}</p>
        <div className="mt-0.5 flex items-center gap-2">
          <Clock className="h-3 w-3 text-text-muted" />
          <span className="text-[11px] text-text-muted">
            {formatRelativeTime(plan.createdAt)}
          </span>
          <span className="text-[11px] text-border">·</span>
          <span
            className={cn(
              "text-[11px] capitalize",
              isComplete
                ? "text-success"
                : isFailed
                  ? "text-destructive"
                  : "text-gold",
            )}
          >
            {plan.status}
          </span>
          <span className="text-[11px] text-border">·</span>
          <span className="truncate text-[11px] text-text-muted">
            {plan.outDir}
          </span>
        </div>
      </div>

      {/* View button — reveal on hover */}
      <button
        onClick={onOpen}
        className="flex shrink-0 items-center gap-1.5 rounded-sm border border-border px-3 py-1.5 text-xs text-text-secondary opacity-0 transition-all group-hover:opacity-100 hover:border-gold/30 hover:text-gold"
      >
        View
        <ArrowRight className="h-3 w-3" />
      </button>
    </div>
  );
}

function EmptyHistory() {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-20 text-center">
      <div className="flex h-14 w-14 items-center justify-center rounded-sm border border-gold/20 bg-gold-subtle">
        <FileText className="h-6 w-6 text-gold-muted" />
      </div>
      <div>
        <p className="text-sm font-medium text-gold">No plans yet</p>
        <p className="mt-1 max-w-xs text-xs text-text-secondary">
          Plans you generate will appear here for review and re-use.
        </p>
      </div>
      <div className="flex items-center gap-1.5 rounded-sm border border-border px-4 py-2 text-[11px] text-text-muted">
        <Clock className="h-3.5 w-3.5" />
        Plans will appear here after your first run
      </div>
    </div>
  );
}
