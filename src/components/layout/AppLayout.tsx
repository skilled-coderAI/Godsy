import { useEffect } from "react";
import { Sidebar } from "./Sidebar";
import { ChatPanel } from "@/components/chat/ChatPanel";
import { PlanViewer } from "@/components/plan/PlanViewer";
import { AgentMonitor } from "@/components/monitor/AgentMonitor";
import { KnowledgeBase } from "@/components/kb/KnowledgeBase";
import { ModelConfig } from "@/components/settings/ModelConfig";
import { PlanHistory } from "@/components/history/PlanHistory";
import { useAppStore } from "@/store";
import { tauriApi } from "@/lib/tauri";

export function AppLayout() {
  const { activeSection, setConfig, setPlans, setKbFiles } = useAppStore();

  // Bootstrap data on mount
  useEffect(() => {
    tauriApi.getConfig().then(setConfig).catch(console.error);
    tauriApi.listPlans().then(setPlans).catch(console.error);
    tauriApi.listKbFiles().then(setKbFiles).catch(console.error);
  }, [setConfig, setPlans, setKbFiles]);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-gold">
      <Sidebar />
      <main className="flex flex-1 flex-col overflow-hidden">
        {activeSection === "chat" && <ChatPanel />}
        {activeSection === "plans" && <PlanViewer />}
        {activeSection === "monitor" && <AgentMonitor />}
        {activeSection === "kb" && <KnowledgeBase />}
        {activeSection === "settings" && <ModelConfig />}
        {activeSection === "history" && <PlanHistory />}
      </main>
    </div>
  );
}
