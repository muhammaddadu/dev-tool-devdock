import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import * as api from "@/lib/api";
import { tildeHome } from "@/lib/format";
import type { ServiceView } from "@/lib/types";

type Props = {
  service: ServiceView;
  open: boolean;
  onClose: () => void;
  onSaved: () => void;
};

/**
 * Save (or edit) a service. When the service already has a `savedId`, we
 * call `updateSavedService` instead of `saveDetectedService` — the form is
 * the same shape, so reusing the dialog is cleaner than duplicating it.
 */
export function SaveServiceDialog({ service, open, onClose, onSaved }: Props) {
  const editing = service.savedId !== null;

  const [label, setLabel] = useState(service.label);
  const [command, setCommand] = useState(service.command ?? "");
  const [ports, setPorts] = useState(service.port?.toString() ?? "");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setLabel(service.label);
      setCommand(service.command ?? "");
      setPorts(service.port?.toString() ?? "");
      setError(null);
    }
  }, [open, service]);

  const submit = async () => {
    setError(null);
    const parsedPorts = parsePorts(ports);

    if (!label.trim()) return setError("Label is required");
    if (!command.trim()) return setError("Command is required");
    if (!service.cwd) return setError("This service has no directory we can save");
    if (parsedPorts.length === 0) {
      return setError("Add at least one port (numbers between 1 and 65535)");
    }

    setSubmitting(true);
    try {
      if (editing && service.savedId) {
        await api.updateSavedService(service.savedId, {
          label: label.trim(),
          command: command.trim(),
          cwd: service.cwd,
          expectedPorts: parsedPorts,
        });
      } else {
        await api.saveDetectedService({
          label: label.trim(),
          command: command.trim(),
          cwd: service.cwd,
          expectedPorts: parsedPorts,
          detectedPid: service.pid,
        });
      }
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} title={editing ? "Edit service" : "Save service"}>
      <div className="space-y-3">
        <Field label="Label">
          <Input
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            autoFocus
          />
        </Field>
        <Field label="Command">
          <Input
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            className="font-mono text-xs"
          />
        </Field>
        <Field label="Directory">
          <p
            className="truncate rounded-md border border-input bg-muted/40 px-2.5 py-1.5 font-mono text-xs text-muted-foreground"
            title={service.cwd ?? ""}
          >
            {tildeHome(service.cwd) || "(none)"}
          </p>
        </Field>
        <Field label="Expected ports">
          <Input
            value={ports}
            onChange={(e) => setPorts(e.target.value)}
            placeholder="e.g. 3000, 3001"
          />
        </Field>
        {!editing && service.pid != null && (
          <p className="text-[11px] text-muted-foreground">
            Detected from PID {service.pid}
            {service.processName ? ` (${service.processName})` : ""}
          </p>
        )}
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
            variant="default"
            size="md"
            onClick={() => void submit()}
            disabled={submitting}
          >
            {submitting ? "Saving…" : "Save"}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1">
      <label className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        {label}
      </label>
      {children}
    </div>
  );
}

function parsePorts(raw: string): number[] {
  return raw
    .split(/[,\s]+/)
    .map((s) => parseInt(s.trim(), 10))
    .filter((n) => Number.isFinite(n) && n > 0 && n < 65536);
}
