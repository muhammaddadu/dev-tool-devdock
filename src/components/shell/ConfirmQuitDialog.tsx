import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import * as api from "@/lib/api";

type Props = {
  open: boolean;
  managedCount: number;
  onClose: () => void;
};

/**
 * Confirmation prompted by the tray's Quit menu when DevDock is currently
 * managing one or more services. Without this, the children would orphan
 * onto launchd and keep running invisibly after the app dies.
 */
export function ConfirmQuitDialog({ open, managedCount, onClose }: Props) {
  const [submitting, setSubmitting] = useState<"stop" | "leave" | null>(null);

  const finish = async (stop: boolean) => {
    setSubmitting(stop ? "stop" : "leave");
    try {
      await api.quitApp(stop);
      // app exits before this returns; if it did return, just close.
      onClose();
    } catch {
      setSubmitting(null);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} title="Quit DevDock?" className="max-w-md">
      <div className="space-y-3">
        <p className="text-sm leading-snug">
          DevDock is running{" "}
          <span className="font-semibold">{managedCount}</span>{" "}
          managed service{managedCount === 1 ? "" : "s"}.
        </p>
        <p className="text-[11px] text-muted-foreground">
          Quitting without stopping leaves them running in the background — you'll have
          to find and kill them yourself next time. Stop &amp; quit sends SIGTERM (same as
          Ctrl-C) and waits up to a few seconds for clean shutdown.
        </p>
        <div className="flex justify-end gap-2 pt-1">
          <Button variant="outline" size="md" onClick={onClose} disabled={submitting !== null}>
            Cancel
          </Button>
          <Button
            variant="outline"
            size="md"
            onClick={() => void finish(false)}
            disabled={submitting !== null}
          >
            {submitting === "leave" ? "Quitting…" : "Quit anyway"}
          </Button>
          <Button
            variant="destructive"
            size="md"
            onClick={() => void finish(true)}
            disabled={submitting !== null}
          >
            {submitting === "stop" ? "Stopping…" : "Stop & quit"}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
