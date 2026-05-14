import { useEffect, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { Folder } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import * as api from "@/lib/api";
import { tildeHome } from "@/lib/format";

type Props = {
  open: boolean;
  onClose: () => void;
  onSaved: () => void;
};

/**
 * Manual entry — save a service the user wants to remember even though it
 * isn't currently running. Same backend command as SaveServiceDialog
 * (`save_detected_service`), just with no pre-fill and a directory picker.
 */
export function AddCommandDialog({ open, onClose, onSaved }: Props) {
  const [label, setLabel] = useState("");
  const [command, setCommand] = useState("");
  const [cwd, setCwd] = useState("");
  const [ports, setPorts] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setLabel("");
      setCommand("");
      setCwd("");
      setPorts("");
      setError(null);
    }
  }, [open]);

  const pickDirectory = async () => {
    try {
      const result = await openDialog({
        directory: true,
        multiple: false,
        title: "Choose project directory",
      });
      if (typeof result === "string" && result.length > 0) {
        setCwd(result);
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const submit = async () => {
    setError(null);
    const parsedPorts = parsePorts(ports);

    if (!label.trim()) return setError("Label is required");
    if (!command.trim()) return setError("Command is required");
    if (!cwd.trim()) return setError("Pick a working directory");
    if (parsedPorts.length === 0) {
      return setError("Add at least one port (numbers between 1 and 65535)");
    }

    setSubmitting(true);
    try {
      await api.saveDetectedService({
        label: label.trim(),
        command: command.trim(),
        cwd: cwd.trim(),
        expectedPorts: parsedPorts,
        detectedPid: null,
      });
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} title="Add Command" className="max-w-md">
      <div className="space-y-3">
        <Field label="Label">
          <Input
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="e.g. Frontend"
            autoFocus
          />
        </Field>
        <Field label="Command">
          <Input
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            placeholder="e.g. pnpm dev"
            className="font-mono text-xs"
          />
        </Field>
        <Field label="Directory">
          <div className="flex items-center gap-2">
            <div
              className="flex-1 truncate rounded-md border border-input bg-muted/40 px-2.5 py-1.5 font-mono text-xs text-muted-foreground"
              title={cwd || ""}
            >
              {cwd ? tildeHome(cwd) : "(choose a folder)"}
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => void pickDirectory()}
              type="button"
            >
              <Folder className="mr-1 h-3 w-3" />
              Browse
            </Button>
          </div>
        </Field>
        <Field label="Expected ports">
          <Input
            value={ports}
            onChange={(e) => setPorts(e.target.value)}
            placeholder="e.g. 3000, 3001"
          />
        </Field>
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
