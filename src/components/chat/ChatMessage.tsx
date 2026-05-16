import { Bot, User } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "@/lib/utils";
import type { ChatMessage as ChatMessageType } from "@/types";

interface Props {
  message: ChatMessageType;
}

export function ChatMessage({ message }: Props) {
  const isUser = message.role === "user";

  if (isUser) {
    return (
      <div className="flex justify-end">
        <div className="flex max-w-[75%] items-start gap-2.5">
          <div className="rounded-sm rounded-tr-none border border-gold/20 bg-gold-subtle px-4 py-3 text-sm text-gold">
            <p className="whitespace-pre-wrap leading-relaxed">{message.content}</p>
          </div>
          <div className="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-sm bg-gold/10 ring-1 ring-gold/30">
            <User className="h-3.5 w-3.5 text-gold" />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex justify-start">
      <div className="flex max-w-[85%] items-start gap-2.5">
        <div className="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-sm bg-surface-elevated ring-1 ring-gold/20">
          <Bot className="h-3.5 w-3.5 text-gold-muted" />
        </div>
        <div
          className={cn(
            "rounded-sm rounded-tl-none border px-4 py-3 text-sm",
            message.type === "error"
              ? "border-destructive/30 bg-destructive/10 text-destructive-foreground"
              : "border-border bg-surface-elevated text-gold",
          )}
        >
          {message.agentName && (
            <p className="mb-1.5 text-[10px] font-medium uppercase tracking-widest text-gold-muted">
              {message.agentName}
            </p>
          )}
          <div className="prose-godsy">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {message.content}
            </ReactMarkdown>
          </div>
        </div>
      </div>
    </div>
  );
}

export function TypingIndicator() {
  return (
    <div className="flex justify-start">
      <div className="flex items-start gap-2.5">
        <div className="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-sm bg-surface-elevated ring-1 ring-gold/20">
          <Bot className="h-3.5 w-3.5 text-gold-muted" />
        </div>
        <div className="rounded-sm rounded-tl-none border border-border bg-surface-elevated px-4 py-3">
          <div className="flex gap-1">
            {[0, 1, 2].map((i) => (
              <span
                key={i}
                className="h-1.5 w-1.5 animate-bounce rounded-full bg-gold-muted"
                style={{ animationDelay: `${i * 150}ms` }}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
