import { useEffect, useRef, useState } from "react";
import {
  Database,
  Upload,
  Trash2,
  FileText,
  FileSpreadsheet,
  File,
  RefreshCw,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import { tauriApi, openFileDialogAndUpload } from "@/lib/tauri";
import { formatBytes, formatRelativeTime } from "@/lib/utils";
import type { KbFile } from "@/types";

function fileIcon(type: string): React.ElementType {
  if (type === "PDF") return FileText;
  if (type === "XLSX" || type === "CSV") return FileSpreadsheet;
  return File;
}

function fileTypeBadgeClass(type: string): string {
  if (type === "PDF") return "text-destructive-foreground border-destructive/20 bg-destructive/10";
  if (type === "XLSX" || type === "CSV")
    return "text-success border-success/20 bg-success/10";
  return "text-text-muted border-border";
}

export function KnowledgeBase() {
  const { kbFiles, setKbFiles, removeKbFile } = useAppStore();
  const [isDragging, setIsDragging] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const refresh = () => {
    tauriApi.listKbFiles().then(setKbFiles).catch(console.error);
  };

  useEffect(() => {
    refresh();
  }, []);

  const performUpload = async (files?: FileList | null) => {
    setUploading(true);
    setError(null);
    try {
      if (files && files.length > 0) {
        // Browser drag-and-drop or file-input path — use file names as mock ids
        for (const f of Array.from(files)) {
          await tauriApi.uploadKbFile(f.name);
        }
      } else {
        // Tauri dialog path (or no-op in browser)
        await openFileDialogAndUpload();
      }
      refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setUploading(false);
    }
  };

  const handleBrowse = () => {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      performUpload();
    } else {
      fileInputRef.current?.click();
    }
  };

  const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    performUpload(e.target.files);
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const handleDelete = async (file: KbFile) => {
    try {
      await tauriApi.deleteKbFile(file.id);
      removeKbFile(file.id);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    if (!e.currentTarget.contains(e.relatedTarget as Node)) {
      setIsDragging(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    if (e.dataTransfer.files.length > 0) {
      performUpload(e.dataTransfer.files);
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* Hidden file input for browser context */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        accept=".pdf,.docx,.xlsx,.md,.csv"
        className="hidden"
        onChange={handleFileInput}
      />

      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-base font-semibold text-gold">Knowledge Base</h1>
          <p className="text-xs text-text-secondary">
            Upload documents to ground planning decisions.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={refresh}
            className="flex items-center gap-1.5 rounded-sm border border-border px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-gold/30 hover:text-gold"
          >
            <RefreshCw className="h-3.5 w-3.5" />
            Refresh
          </button>
          <button
            onClick={handleBrowse}
            disabled={uploading}
            className="flex items-center gap-1.5 rounded-sm border border-gold/30 bg-gold-subtle px-3 py-1.5 text-xs font-medium text-gold transition-all hover:bg-gold hover:text-background disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Upload className="h-3.5 w-3.5" />
            {uploading ? "Uploading…" : "Add Files"}
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-y-auto px-6 py-6">
        {/* Drop zone */}
        <div
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          className={cn(
            "mb-6 flex flex-col items-center justify-center rounded-sm border-2 border-dashed py-12 transition-all duration-200",
            isDragging
              ? "border-gold/60 bg-gold-subtle/50 shadow-[0_0_24px_rgba(212,175,55,0.08)]"
              : "border-border bg-surface hover:border-gold/25 hover:bg-surface-elevated",
          )}
        >
          <div
            className={cn(
              "mb-4 flex h-12 w-12 items-center justify-center rounded-sm ring-1 transition-all",
              isDragging
                ? "bg-gold-subtle ring-gold/30"
                : "bg-surface-elevated ring-border",
            )}
          >
            <Database
              className={cn(
                "h-5 w-5 transition-colors",
                isDragging ? "text-gold" : "text-text-muted",
              )}
            />
          </div>
          <p className="text-sm font-medium text-text-secondary">
            Drop files here or{" "}
            <button
              onClick={handleBrowse}
              className="text-gold underline-offset-2 hover:underline"
            >
              browse
            </button>
          </p>
          <p className="mt-1.5 text-[11px] text-text-muted">
            PDF · DOCX · XLSX · MD supported
          </p>
        </div>

        {/* Error */}
        {error && (
          <div className="mb-4 flex items-start gap-2 rounded-sm border border-destructive/30 bg-destructive/10 px-4 py-2.5 text-xs text-destructive-foreground">
            <span className="mt-0.5 shrink-0">⚠</span>
            <span>{error}</span>
          </div>
        )}

        {/* File list */}
        {kbFiles.length === 0 ? (
          <EmptyState onUpload={handleBrowse} />
        ) : (
          <div>
            <p className="mb-3 text-[10px] uppercase tracking-widest text-text-muted">
              {kbFiles.length}{" "}
              {kbFiles.length === 1 ? "document" : "documents"}
            </p>
            <div className="space-y-2">
              {kbFiles.map((f) => (
                <FileRow
                  key={f.id}
                  file={f}
                  onDelete={() => handleDelete(f)}
                />
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function FileRow({
  file,
  onDelete,
}: {
  file: KbFile;
  onDelete: () => void;
}) {
  const Icon = fileIcon(file.fileType);
  return (
    <div className="group flex items-center gap-4 rounded-sm border border-border bg-surface px-4 py-3 transition-colors hover:border-gold/20 hover:bg-surface-elevated">
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-sm bg-surface-elevated ring-1 ring-border">
        <Icon className="h-4 w-4 text-gold-muted" />
      </div>

      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-gold">{file.name}</p>
        <div className="mt-0.5 flex items-center gap-2">
          <span
            className={cn(
              "rounded-xs border px-1.5 py-0.5 text-[10px] font-medium",
              fileTypeBadgeClass(file.fileType),
            )}
          >
            {file.fileType}
          </span>
          <span className="text-[11px] text-text-muted">
            {formatBytes(file.size)}
          </span>
          {file.addedAt && (
            <span className="text-[11px] text-text-muted">
              · {formatRelativeTime(file.addedAt)}
            </span>
          )}
        </div>
      </div>

      <button
        onClick={onDelete}
        title="Remove from knowledge base"
        className="rounded-sm p-1.5 text-text-muted opacity-0 transition-all group-hover:opacity-100 hover:text-destructive"
      >
        <Trash2 className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}

function EmptyState({ onUpload }: { onUpload: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-12 text-center">
      <div className="flex h-14 w-14 items-center justify-center rounded-sm border border-gold/20 bg-gold-subtle">
        <Database className="h-6 w-6 text-gold-muted" />
      </div>
      <div>
        <p className="text-sm font-medium text-gold">No documents yet</p>
        <p className="mt-1 max-w-xs text-xs text-text-secondary">
          Upload SOPs, spreadsheets, and prior plans to ground your architecture
          decisions with real business context.
        </p>
      </div>
      <button
        onClick={onUpload}
        className="flex items-center gap-2 rounded-sm border border-gold/30 bg-gold-subtle px-4 py-2 text-xs font-medium text-gold transition-all hover:bg-gold hover:text-background"
      >
        <Upload className="h-3.5 w-3.5" />
        Browse Files
      </button>
    </div>
  );
}
