import { useRef, useState, type KeyboardEvent } from "react";
import { Send, Lightbulb } from "lucide-react";
import { cn } from "@/lib/utils";

const QUICK_PROMPTS = [
  "Track truck deliveries and driver assignments",
  "Manage inventory across multiple warehouses",
  "Digital staff attendance and payroll system",
  "Customer order tracking for a retail shop",
];

interface Props {
  onSubmit: (value: string) => void;
  disabled?: boolean;
}

export function ChatInput({ onSubmit, disabled }: Props) {
  const [value, setValue] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = () => {
    const trimmed = value.trim();
    if (!trimmed || disabled) return;
    onSubmit(trimmed);
    setValue("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleInput = () => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 160)}px`;
  };

  return (
    <div className="border-t border-border bg-surface px-4 py-4">
      {/* Quick prompts */}
      <div className="mb-3 flex flex-wrap gap-1.5">
        {QUICK_PROMPTS.map((p) => (
          <button
            key={p}
            onClick={() => {
              setValue(p);
              textareaRef.current?.focus();
            }}
            disabled={disabled}
            className="flex items-center gap-1.5 rounded-sm border border-border px-2.5 py-1 text-[11px] text-text-secondary transition-colors hover:border-gold/30 hover:bg-gold-subtle hover:text-gold disabled:opacity-40"
          >
            <Lightbulb className="h-3 w-3" />
            {p}
          </button>
        ))}
      </div>

      {/* Input row */}
      <div
        className={cn(
          "flex items-end gap-3 rounded-sm border bg-surface-elevated px-3 py-2 transition-colors",
          disabled ? "border-border/50 opacity-60" : "border-border focus-within:border-gold/40 focus-within:shadow-[0_0_12px_rgba(212,175,55,0.08)]",
        )}
      >
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          onInput={handleInput}
          disabled={disabled}
          placeholder="Describe your business problem… (Shift+Enter for newline)"
          rows={1}
          className="flex-1 resize-none bg-transparent text-sm text-gold placeholder:text-text-muted focus:outline-none disabled:cursor-not-allowed"
          style={{ minHeight: "36px" }}
        />
        <button
          onClick={handleSubmit}
          disabled={disabled || !value.trim()}
          className={cn(
            "mb-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-sm transition-all",
            value.trim() && !disabled
              ? "bg-gold text-background hover:bg-gold-hover shadow-[0_0_10px_rgba(212,175,55,0.25)]"
              : "bg-surface text-text-muted cursor-not-allowed",
          )}
        >
          <Send className="h-3.5 w-3.5" />
        </button>
      </div>
      <p className="mt-1.5 text-[10px] text-text-muted">
        Enter to send · Shift+Enter for new line · Godsy plans it, one coding agent ships it.
      </p>
    </div>
  );
}
