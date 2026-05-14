import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import * as api from "@/lib/api";
import { tildeHome } from "@/lib/format";
import type { ServiceView } from "@/lib/types";

type Props = {
  service: ServiceView;
  open: boolean;
  onClose: () => void;
  onKilled: () => void;
};

/**
 * Confirmation modal for terminating a process DevDock didn't start. Per the
 * security rule in AGENTS.md, every destructive action shows PID, command,
 * and cwd so the user knows exactly what they're signaling.
 */
export function KillProcessDialog({ service, open, onClose, onKilled }: Props) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const pid = service.pid;
  if (pid == null) return null;

  const kill = async () => {
    setError(null);
    setSubmitting(true);
    try {
      await api.killDetectedProcess(pid);
      onKilled();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} title="Kill external process?">
      <div className="space-y-3">
        <Detail label="Port" value={service.port != null ? `:${service.port}` : null} mono />
        <Detail label="PID" value={String(pid)} mono />
        <Detail
          label="Process"
          value={service.processName ?? service.label}
        />
        {service.command && (
          <Detail label="Command" value={service.command} mono multiline />
        )}
        {service.cwd && (
          <Detail label="Directory" value={tildeHome(service.cwd)} mono />
        )}

        <p className="text-[11px] text-muted-foreground">
          {service.savedId
            ? "This service is saved, but DevDock didn't start it. Sending SIGTERM frees the port so you can Run it through DevDock instead."
            : "This process was not started by DevDock. SIGTERM is sent — the same signal Ctrl-C uses in a terminal."}
        </p>

        {error && (
          <p className="text-[11px] text-destructive" role="alert">
            {error}
          </p>
        )}

        <div className="flex justify-end gap-2 pt-1">
          <Button variant="outline" size="md" onClick={onClose} disabled={submitting}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            size="md"
            onClick={() => void kill()}
            disabled={submitting}
          >
            {submitting ? "Sending…" : "Kill Process"}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

function Detail({
  label,
  value,
  mono = false,
  multiline = false,
}: {
  label: string;
  value: string | null;
  mono?: boolean;
  multiline?: boolean;
}) {
  if (value == null || value === "") return null;
  return (
    <div className="flex items-start gap-3 text-xs">
      <span className="w-20 shrink-0 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        {label}
      </span>
      <span
        className={`flex-1 ${mono ? "font-mono" : ""} ${
          multiline ? "break-all" : "truncate"
        }`}
        title={value}
      >
        {value}
      </span>
    </div>
  );
}
