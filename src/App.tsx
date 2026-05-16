import { AppLayout } from "@/components/layout/AppLayout";
import { Toaster } from "sonner";

export function App() {
  return (
    <>
      <AppLayout />
      <Toaster
        theme="dark"
        position="bottom-right"
        toastOptions={{
          style: {
            background: "var(--surface-elevated)",
            border: "1px solid var(--border)",
            color: "var(--gold)",
          },
        }}
      />
    </>
  );
}
