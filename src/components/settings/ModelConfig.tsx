import { useState, useEffect } from "react";
import { Save, Check, AlertCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import { tauriApi } from "@/lib/tauri";
import type { GodsyConfig } from "@/types";

// ─── Field layout ──────────────────────────────────────────────────────────────

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="mb-8">
      <p className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-gold-muted">
        {title}
      </p>
      <div className="rounded-sm border border-border bg-surface divide-y divide-border">
        {children}
      </div>
    </section>
  );
}

function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="grid grid-cols-[200px_1fr] items-start gap-4 px-5 py-4">
      <div className="pt-0.5">
        <p className="text-sm font-medium text-gold">{label}</p>
        {hint && (
          <p className="mt-0.5 text-[11px] leading-relaxed text-text-muted">
            {hint}
          </p>
        )}
      </div>
      <div>{children}</div>
    </div>
  );
}

// ─── Input atoms ───────────────────────────────────────────────────────────────

function TextInput({
  value,
  onChange,
  placeholder,
  type = "text",
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  type?: string;
}) {
  return (
    <input
      type={type}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      autoComplete="off"
      className="w-full rounded-sm border border-border bg-surface-elevated px-3 py-2 text-sm text-gold placeholder:text-text-muted focus:border-gold/40 focus:shadow-[0_0_10px_rgba(212,175,55,0.06)] focus:outline-none transition-colors"
    />
  );
}

function SelectInput({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="w-full rounded-sm border border-border bg-surface-elevated px-3 py-2 text-sm text-gold focus:border-gold/40 focus:outline-none transition-colors"
    >
      {options.map((o) => (
        <option key={o.value} value={o.value} className="bg-surface-elevated text-gold">
          {o.label}
        </option>
      ))}
    </select>
  );
}

// ─── Main component ────────────────────────────────────────────────────────────

export function ModelConfig() {
  const { config, setConfig } = useAppStore();
  const [form, setForm] = useState<GodsyConfig | null>(null);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (config) setForm(config);
  }, [config]);

  const set = <K extends keyof GodsyConfig>(key: K, value: GodsyConfig[K]) => {
    setForm((prev) => (prev ? { ...prev, [key]: value } : prev));
  };

  const handleSave = async () => {
    if (!form) return;
    setSaving(true);
    setError(null);
    try {
      await tauriApi.saveConfig(form);
      setConfig(form);
      setSaved(true);
      setTimeout(() => setSaved(false), 2500);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  if (!form) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-gold/20 border-t-gold" />
      </div>
    );
  }

  const isCloud = form.provider === "ollama_cloud" || form.provider === "cloudflare_workers";
  const isVane = form.grounding === "vane";

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-base font-semibold text-gold">
            Model Configuration
          </h1>
          <p className="text-xs text-text-secondary">
            Local and cloud inference settings.
          </p>
        </div>
        <button
          onClick={handleSave}
          disabled={saving}
          className={cn(
            "flex items-center gap-1.5 rounded-sm border px-4 py-1.5 text-xs font-medium transition-all disabled:opacity-50",
            saved
              ? "border-success/30 bg-success/10 text-success"
              : "border-gold/30 bg-gold-subtle text-gold hover:bg-gold hover:text-background",
          )}
        >
          {saved ? (
            <Check className="h-3.5 w-3.5" />
          ) : (
            <Save className="h-3.5 w-3.5" />
          )}
          {saved ? "Saved!" : saving ? "Saving…" : "Save"}
        </button>
      </header>

      <div className="flex-1 overflow-y-auto px-6 py-6">
        <div className="mx-auto max-w-2xl">
          {/* Error banner */}
          {error && (
            <div className="mb-6 flex items-start gap-2 rounded-sm border border-destructive/30 bg-destructive/10 px-4 py-3 text-xs text-destructive-foreground">
              <AlertCircle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {/* Inference */}
          <Section title="Inference">
            <Field label="Provider" hint="Local Ollama keeps data on-device">
              <SelectInput
                value={form.provider}
                onChange={(v) => set("provider", v as GodsyConfig["provider"])}
                options={[
                  { value: "ollama", label: "Ollama (local)" },
                  { value: "ollama_cloud", label: "Ollama Cloud (hosted)" },
                  { value: "cloudflare_workers", label: "Cloudflare Workers AI" },
                ]}
              />
            </Field>
            <Field label="Model" hint="Planning model (not a code generator)">
              <TextInput
                value={form.model}
                onChange={(v) => set("model", v)}
                placeholder="qwen2.5"
              />
            </Field>
            <Field label="Base URL" hint="Ollama endpoint">
              <TextInput
                value={form.modelUrl}
                onChange={(v) => set("modelUrl", v)}
                placeholder="http://localhost:11434"
              />
            </Field>
            {isCloud && (
              <Field label="API Key" hint="Bearer token for hosted inference">
                <TextInput
                  type="password"
                  value={form.apiKey}
                  onChange={(v) => set("apiKey", v)}
                  placeholder="••••••••••••••••"
                />
              </Field>
            )}
          </Section>

          {/* Web Grounding */}
          <Section title="Web Grounding">
            <Field
              label="Gateway"
              hint="Vane wraps SearXNG + Ollama for cited answers"
            >
              <SelectInput
                value={form.grounding}
                onChange={(v) =>
                  set("grounding", v as GodsyConfig["grounding"])
                }
                options={[
                  { value: "none", label: "None (offline)" },
                  { value: "vane", label: "Vane / Perplexica (local)" },
                ]}
              />
            </Field>
            {isVane && (
              <Field label="Vane URL" hint="Default: http://localhost:3000">
                <TextInput
                  value={form.groundingUrl}
                  onChange={(v) => set("groundingUrl", v)}
                  placeholder="http://localhost:3000"
                />
              </Field>
            )}
          </Section>

          {/* Output */}
          <Section title="Output">
            <Field label="Output Directory" hint="Plan bundles are written here">
              <TextInput
                value={form.outDir}
                onChange={(v) => set("outDir", v)}
                placeholder="./godsy-plans"
              />
            </Field>
            <Field
              label="Confidence Threshold"
              hint={`Plans below this score trigger a Validator re-run (${(form.confidenceThreshold * 100).toFixed(0)}%)`}
            >
              <div className="flex items-center gap-3">
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={form.confidenceThreshold}
                  onChange={(e) =>
                    set("confidenceThreshold", parseFloat(e.target.value))
                  }
                  className="flex-1"
                />
                <span className="w-10 shrink-0 text-right text-sm tabular-nums text-gold">
                  {form.confidenceThreshold.toFixed(2)}
                </span>
              </div>
              <div className="mt-2 flex justify-between text-[10px] text-text-muted">
                <span>Lenient</span>
                <span>Strict</span>
              </div>
            </Field>
          </Section>

          {/* Info box */}
          <div className="rounded-sm border border-border bg-surface px-5 py-4 text-[11px] leading-relaxed text-text-muted">
            <span className="font-semibold text-gold-muted">Security note:</span>{" "}
            API keys are stored in your OS keychain when saved via the Tauri
            shell. In CLI mode, use <code className="text-gold-muted">GODSY_API_KEY</code> or{" "}
            <code className="text-gold-muted">OLLAMA_API_KEY</code> environment variables.
          </div>
        </div>
      </div>
    </div>
  );
}
