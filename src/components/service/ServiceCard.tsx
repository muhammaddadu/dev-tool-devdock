import { useEffect, useState } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusDot } from "@/components/ui/status-dot";
import { KillProcessDialog } from "./KillProcessDialog";
import { LogsDialog } from "./LogsDialog";
import { SaveServiceDialog } from "./SaveServiceDialog";
import { ServiceMenu, type MenuAction } from "./ServiceMenu";
import { useNowTick } from "@/hooks/useNowTick";
import * as api from "@/lib/api";
import { cn } from "@/lib/cn";
import { buildPasteCommand, formatAge, formatPort, tildeHome } from "@/lib/format";
import { useAppStore } from "@/lib/store";
import type { ServiceView } from "@/lib/types";

type Props = {
  service: ServiceView;
  compact?: boolean;
};

export function ServiceCard({ service, compact = false }: Props) {
  const runService = useAppStore((s) => s.runService);
  const stopService = useAppStore((s) => s.stopService);
  const restartService = useAppStore((s) => s.restartService);
  const refresh = useAppStore((s) => s.refresh);
  const editors = useAppStore((s) => s.editors);

  const [copied, setCopied] = useState(false);
  const [saveOpen, setSaveOpen] = useState(false);
  const [killOpen, setKillOpen] = useState(false);
  const [logsOpen, setLogsOpen] = useState(false);
  useEffect(() => {
    if (!copied) return;
    const t = window.setTimeout(() => setCopied(false), 1200);
    return () => window.clearTimeout(t);
  }, [copied]);

  const now = useNowTick();
  const age = formatAge(service.startedAtUnix, now);

  const copy = async (text: string) => {
    try {
      await api.copyToClipboard(text);
      setCopied(true);
    } catch {
      // Silent — the menu is best-effort. Future: surface a toast.
    }
  };

  const open = async () => {
    if (service.url) {
      await api.openUrl(service.url).catch(() => {});
    }
  };

  const actions: MenuAction[] = [];
  if (service.logPath) {
    actions.push({ label: "View Logs", onSelect: () => setLogsOpen(true) });
  }
  if (service.url) {
    actions.push({ label: "Copy URL", onSelect: () => copy(service.url!) });
  }
  if (service.command) {
    // Paste-into-terminal form: prepend `cd <quoted-cwd> && …` so the user
    // can drop it into Terminal and re-run from the right directory.
    actions.push({
      label: "Copy command",
      onSelect: () => copy(buildPasteCommand(service.command!, service.cwd)),
    });
  }
  if (service.cwd) {
    const cwd = service.cwd;
    // One "Open in <editor>" entry per detected editor. If none are installed,
    // nothing shows — the user isn't taunted by an action that can't work.
    for (const editor of editors) {
      actions.push({
        label: `Open in ${editor.displayName}`,
        onSelect: () => api.openInEditor(editor.id, cwd).catch(() => {}),
      });
    }
    actions.push({
      label: "Open in Terminal",
      onSelect: () => api.openInTerminal(cwd).catch(() => {}),
    });
    actions.push({
      label: "Reveal in Finder",
      onSelect: () => api.openPath(cwd).catch(() => {}),
    });
  }

  if (service.savedId) {
    const savedId = service.savedId;
    actions.push({
      label: "Edit",
      onSelect: () => setSaveOpen(true),
    });
    actions.push({
      label: service.pinned ? "Unpin" : "Pin",
      onSelect: async () => {
        try {
          await api.setServicePinned(savedId, !service.pinned);
          await refresh();
        } catch {
          // ignore
        }
      },
    });
    actions.push({
      label: "Forget service",
      destructive: true,
      onSelect: async () => {
        try {
          await api.unsaveService(savedId);
          await refresh();
        } catch {
          // ignore
        }
      },
    });
  }

  return (
    <div
      className={cn(
        "group rounded-lg border border-border bg-card transition-colors",
        compact ? "px-2.5 py-1.5" : "px-3 py-2",
        "hover:border-foreground/20 hover:bg-card/80",
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <StatusDot status={service.status} />
            <span
              className={cn(
                "truncate font-medium",
                compact ? "text-xs" : "text-sm",
              )}
            >
              {service.label}
            </span>
            <div className="ml-auto flex items-center gap-1">
              {age && !compact && (
                <span
                  className="text-[10px] tabular-nums text-muted-foreground/70"
                  title="Process started"
                >
                  {age}
                </span>
              )}
              {service.port != null && (
                <span className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] tabular-nums text-muted-foreground">
                  {formatPort(service.port)}
                </span>
              )}
            </div>
          </div>
          {!compact && service.command && (
            <p
              className="mt-1 truncate font-mono text-[11px] text-muted-foreground"
              title={service.command}
            >
              {service.command}
            </p>
          )}
          {!compact && service.cwd && (
            <p
              className="truncate text-[11px] text-muted-foreground/80"
              title={service.cwd}
            >
              {tildeHome(service.cwd)}
            </p>
          )}
        </div>
      </div>

      <div
        className={cn(
          "flex flex-wrap items-center gap-1",
          compact ? "mt-1.5" : "mt-2",
        )}
      >
        {service.canOpen && (
          <Button variant="outline" size="sm" onClick={open}>
            Open
          </Button>
        )}
        {service.url && (
          <Button
            variant="ghost"
            size="icon"
            aria-label={copied ? "URL copied" : "Copy URL"}
            title={copied ? "Copied" : "Copy URL"}
            onClick={() => copy(service.url!)}
          >
            {copied ? (
              <Check className="h-3.5 w-3.5 text-[hsl(var(--success))]" />
            ) : (
              <Copy className="h-3.5 w-3.5" />
            )}
          </Button>
        )}
        {service.canRun && (
          <Button variant="default" size="sm" onClick={() => void runService(service.id)}>
            Run
          </Button>
        )}
        {service.canRestart && (
          <Button variant="outline" size="sm" onClick={() => void restartService(service.id)}>
            Restart
          </Button>
        )}
        {service.canStop && (
          <Button variant="outline" size="sm" onClick={() => void stopService(service.id)}>
            Stop
          </Button>
        )}
        {service.canSave && (
          <Button variant="outline" size="sm" onClick={() => setSaveOpen(true)}>
            Save
          </Button>
        )}
        {service.canKill && (
          <Button variant="destructive" size="sm" onClick={() => setKillOpen(true)}>
            Kill
          </Button>
        )}
        <div className="ml-auto">
          <ServiceMenu actions={actions} />
        </div>
      </div>
      <SaveServiceDialog
        service={service}
        open={saveOpen}
        onClose={() => setSaveOpen(false)}
        onSaved={() => void refresh()}
      />
      <KillProcessDialog
        service={service}
        open={killOpen}
        onClose={() => setKillOpen(false)}
        onKilled={() => void refresh()}
      />
      <LogsDialog
        service={service}
        open={logsOpen}
        onClose={() => setLogsOpen(false)}
      />
    </div>
  );
}
