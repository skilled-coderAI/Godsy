import {
  MessageSquare,
  FileText,
  Activity,
  Database,
  Settings,
  Clock,
  ChevronLeft,
  ChevronRight,
  Sparkles,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import type { NavSection } from "@/types";

interface NavItem {
  key: NavSection;
  label: string;
  icon: React.ElementType;
  title: string;
}

const NAV_ITEMS: NavItem[] = [
  { key: "chat", label: "Chat", icon: MessageSquare, title: "Plan with Godsy" },
  { key: "plans", label: "Plans", icon: FileText, title: "Plan Viewer" },
  { key: "monitor", label: "Monitor", icon: Activity, title: "Agent Monitor" },
  { key: "kb", label: "Knowledge", icon: Database, title: "Knowledge Base" },
  { key: "history", label: "History", icon: Clock, title: "Plan History" },
];

const BOTTOM_ITEMS: NavItem[] = [
  { key: "settings", label: "Settings", icon: Settings, title: "Model Configuration" },
];

export function Sidebar() {
  const { activeSection, sidebarExpanded, setActiveSection, toggleSidebar } =
    useAppStore();

  return (
    <aside
      className={cn(
        "flex h-full flex-col border-r border-border bg-surface transition-all duration-300",
        sidebarExpanded ? "w-60" : "w-16",
      )}
    >
      {/* Logo */}
      <div
        className={cn(
          "flex items-center border-b border-border px-3 py-5 transition-all duration-300",
          sidebarExpanded ? "gap-3" : "justify-center",
        )}
      >
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-sm bg-gold/10 ring-1 ring-gold/30">
          <Sparkles className="h-4 w-4 text-gold" />
        </div>
        {sidebarExpanded && (
          <div className="overflow-hidden">
            <p className="truncate text-sm font-semibold tracking-wide text-gold">
              Godsy
            </p>
            <p className="truncate text-[10px] uppercase tracking-widest text-text-secondary">
              Architecture Studio
            </p>
          </div>
        )}
      </div>

      {/* Primary nav */}
      <nav className="flex flex-1 flex-col gap-0.5 overflow-y-auto px-2 py-3">
        {NAV_ITEMS.map((item) => (
          <NavButton
            key={item.key}
            item={item}
            isActive={activeSection === item.key}
            expanded={sidebarExpanded}
            onClick={() => setActiveSection(item.key)}
          />
        ))}
      </nav>

      {/* Bottom nav */}
      <div className="flex flex-col gap-0.5 border-t border-border px-2 py-3">
        {BOTTOM_ITEMS.map((item) => (
          <NavButton
            key={item.key}
            item={item}
            isActive={activeSection === item.key}
            expanded={sidebarExpanded}
            onClick={() => setActiveSection(item.key)}
          />
        ))}

        {/* Collapse toggle */}
        <button
          onClick={toggleSidebar}
          className={cn(
            "mt-1 flex items-center rounded-sm border border-border px-2 py-2 text-text-secondary transition-colors hover:bg-gold-subtle hover:text-gold",
            sidebarExpanded ? "gap-2" : "justify-center",
          )}
          title={sidebarExpanded ? "Collapse sidebar" : "Expand sidebar"}
        >
          {sidebarExpanded ? (
            <>
              <ChevronLeft className="h-4 w-4 shrink-0" />
              <span className="text-xs">Collapse</span>
            </>
          ) : (
            <ChevronRight className="h-4 w-4 shrink-0" />
          )}
        </button>
      </div>
    </aside>
  );
}

interface NavButtonProps {
  item: NavItem;
  isActive: boolean;
  expanded: boolean;
  onClick: () => void;
}

function NavButton({ item, isActive, expanded, onClick }: NavButtonProps) {
  const Icon = item.icon;
  return (
    <button
      onClick={onClick}
      title={!expanded ? item.title : undefined}
      className={cn(
        "group relative flex w-full items-center rounded-sm px-2 py-2.5 text-sm font-medium transition-all duration-150",
        expanded ? "gap-3" : "justify-center",
        isActive
          ? "bg-gold-subtle text-gold ring-1 ring-gold/20"
          : "text-text-secondary hover:bg-gold-subtle/50 hover:text-gold",
      )}
    >
      <Icon
        className={cn(
          "h-4 w-4 shrink-0 transition-colors",
          isActive ? "text-gold" : "text-text-secondary group-hover:text-gold",
        )}
      />
      {expanded && <span className="truncate">{item.label}</span>}
    </button>
  );
}
