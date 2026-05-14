import { useMemo, useRef, useState } from "react";
import { RefreshCw, Settings as SettingsIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ThemeToggle } from "./ThemeToggle";
import { AddCommandDialog } from "@/components/service/AddCommandDialog";
import { ServiceSection } from "@/components/service/ServiceSection";
import { CollapsibleSection } from "@/components/service/CollapsibleSection";
import { EmptyState } from "@/components/service/EmptyState";
import { ForegroundQuiet } from "@/components/service/ForegroundQuiet";
import { SettingsDialog } from "@/components/settings/SettingsDialog";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useAppStore } from "@/lib/store";
import type { ServiceView } from "@/lib/types";

function matches(s: ServiceView, q: string) {
  if (!q) return true;
  const needle = q.toLowerCase();
  return (
    s.label.toLowerCase().includes(needle) ||
    (s.command ?? "").toLowerCase().includes(needle) ||
    (s.cwd ?? "").toLowerCase().includes(needle) ||
    (s.projectName ?? "").toLowerCase().includes(needle) ||
    String(s.port ?? "").includes(needle)
  );
}

export function TrayPanel() {
  const services = useAppStore((s) => s.services);
  const searchQuery = useAppStore((s) => s.searchQuery);
  const setSearchQuery = useAppStore((s) => s.setSearchQuery);
  const isRefreshing = useAppStore((s) => s.isRefreshing);
  const refresh = useAppStore((s) => s.refresh);
  const searchRef = useRef<HTMLInputElement>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [addOpen, setAddOpen] = useState(false);

  useKeyboardShortcuts({
    onRefresh: refresh,
    onFocusSearch: () => searchRef.current?.focus(),
  });

  const { dev, tooling, system, pinned, recent } = useMemo(() => {
    const filtered = services.filter((s) => matches(s, searchQuery));
    const running = filtered.filter(
      (s) => s.status === "running" || s.status === "starting",
    );
    return {
      dev: running.filter((s) => s.bucket === "dev"),
      tooling: running.filter((s) => s.bucket === "tooling"),
      system: running.filter((s) => s.bucket === "system"),
      pinned: filtered.filter((s) => s.pinned && s.status !== "running"),
      recent: filtered.filter((s) => !s.pinned && s.status !== "running"),
    };
  }, [services, searchQuery]);

  const hasAny = services.length > 0;
  const hasDev = dev.length > 0 || pinned.length > 0 || recent.length > 0;
  const backgroundCount = tooling.length + system.length;
  const searchActive = searchQuery.length > 0;

  return (
    // The Tauri window is borderless and its NSWindow background was forced
    // clear in the setup hook (see `force_clear_window_background`). The
    // panel below is opaque with rounded corners — the corners cut through
    // to the desktop because the window outside the panel is genuinely
    // transparent. No vibrancy, no CSS backdrop-filter; just a clean
    // floating rounded card.
    <div className="flex h-screen w-screen flex-col">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-2xl border border-border bg-background text-foreground">
      <header className="flex items-center justify-between border-b border-border px-3 py-2">
        <div className="flex items-center gap-2">
          <svg
            viewBox="0 0 64 64"
            className="h-4 w-4 text-foreground/85"
            aria-hidden="true"
          >
            <circle
              cx="32"
              cy="32"
              r="29"
              fill="none"
              stroke="currentColor"
              strokeOpacity="0.45"
              strokeWidth="3"
            />
            <circle cx="32" cy="32" r="20" fill="none" stroke="currentColor" strokeWidth="3.5" />
            <circle cx="32" cy="32" r="6" fill="currentColor" />
          </svg>
          <span className="text-sm font-semibold tracking-tight">DevDock</span>
        </div>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => void refresh()}
            aria-label="Refresh"
            title="Refresh"
            disabled={isRefreshing}
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
          </Button>
          <ThemeToggle />
          <Button
            variant="ghost"
            size="icon"
            aria-label="Settings"
            title="Settings"
            onClick={() => setSettingsOpen(true)}
          >
            <SettingsIcon className="h-3.5 w-3.5" />
          </Button>
        </div>
      </header>

      <div className="border-b border-border p-2">
        <Input
          ref={searchRef}
          placeholder="Search services, ports, projects..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      <main className="flex-1 overflow-y-auto">
        {!hasAny && <EmptyState onAddCommand={() => setAddOpen(true)} />}
        {hasAny && (
          <div className="flex flex-col gap-1 p-2">
            {!hasDev && !searchActive && tooling.length + system.length > 0 && (
              <ForegroundQuiet backgroundCount={backgroundCount} />
            )}
            <ServiceSection title="Running" services={dev} />
            <ServiceSection title="Pinned" services={pinned} />
            <ServiceSection title="Recently Seen" services={recent} />
            {/* Tooling auto-expands when there's nothing in the foreground —
                otherwise the user lands on an empty-feeling panel. System
                stays collapsed unless the search needs it. */}
            <CollapsibleSection
              title="Tooling"
              services={tooling}
              defaultOpen={searchActive || (!hasDev && tooling.length > 0)}
              compact
            />
            <CollapsibleSection
              title="System"
              services={system}
              defaultOpen={searchActive}
              compact
            />
          </div>
        )}
      </main>

      <footer className="flex items-center gap-2 border-t border-border px-2 py-2">
        <Button variant="outline" size="sm" onClick={() => setAddOpen(true)}>
          Add Command
        </Button>
      </footer>
      </div>

      <SettingsDialog open={settingsOpen} onClose={() => setSettingsOpen(false)} />
      <AddCommandDialog
        open={addOpen}
        onClose={() => setAddOpen(false)}
        onSaved={() => void refresh()}
      />
    </div>
  );
}
