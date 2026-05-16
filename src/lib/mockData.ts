import type { GodsyConfig, PlanBundle, KbFile } from "@/types";

export const MOCK_CONFIG: GodsyConfig = {
  provider: "ollama",
  model: "qwen2.5",
  modelUrl: "http://localhost:11434",
  apiKey: "",
  grounding: "none",
  groundingUrl: "",
  outDir: "./godsy-plans",
  confidenceThreshold: 0.75,
};

export const MOCK_PLANS: PlanBundle[] = [
  {
    id: "plan-20260516-001",
    title: "Truck Delivery Tracker",
    createdAt: "2026-05-16T09:30:00Z",
    status: "complete",
    outDir: "./godsy-plans/plan-20260516-001",
  },
  {
    id: "plan-20260515-002",
    title: "Inventory Management System",
    createdAt: "2026-05-15T14:22:00Z",
    status: "complete",
    outDir: "./godsy-plans/plan-20260515-002",
  },
  {
    id: "plan-20260514-003",
    title: "Staff Attendance Portal",
    createdAt: "2026-05-14T11:05:00Z",
    status: "complete",
    outDir: "./godsy-plans/plan-20260514-003",
  },
];

export const MOCK_KB_FILES: KbFile[] = [
  {
    id: "sop-delivery.pdf",
    name: "SOP-Delivery-Operations.pdf",
    size: 452_000,
    fileType: "PDF",
    addedAt: "2026-05-14",
  },
  {
    id: "inventory-sheet.xlsx",
    name: "Inventory-Master.xlsx",
    size: 128_000,
    fileType: "XLSX",
    addedAt: "2026-05-13",
  },
];
