import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { ConfirmQuitDialog } from "@/components/shell/ConfirmQuitDialog";
import { TrayPanel } from "@/components/shell/TrayPanel";
import { useAutoRefresh } from "@/hooks/useAutoRefresh";
import { useAppStore } from "@/lib/store";

export function App() {
  const refresh = useAppStore((s) => s.refresh);
  const loadEditors = useAppStore((s) => s.loadEditors);
  const loadSettings = useAppStore((s) => s.loadSettings);
  const refreshIntervalMs = useAppStore((s) => s.refreshIntervalMs);

  const [quitPrompt, setQuitPrompt] = useState<{ count: number } | null>(null);

  useEffect(() => {
    void loadEditors();
    void loadSettings();
  }, [loadEditors, loadSettings]);

  // The tray's Quit menu only emits this event when DevDock is actively
  // managing services. Otherwise it exits immediately on the backend.
  useEffect(() => {
    const unlistenPromise = listen<number>("devdock://confirm-quit", (event) => {
      setQuitPrompt({ count: event.payload ?? 0 });
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useAutoRefresh(refresh, refreshIntervalMs);

  return (
    <>
      <TrayPanel />
      <ConfirmQuitDialog
        open={quitPrompt !== null}
        managedCount={quitPrompt?.count ?? 0}
        onClose={() => setQuitPrompt(null)}
      />
    </>
  );
}
