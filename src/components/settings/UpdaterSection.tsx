import { useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { Button } from "@/components/ui/button";

type State =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "no-update"; version: string }
  | { kind: "available"; update: Update }
  | { kind: "installing"; downloaded: number; total: number | null }
  | { kind: "ready-to-restart" }
  | { kind: "error"; message: string };

/**
 * Manual update check inside the Settings dialog. We don't auto-check at app
 * startup yet — once the user trusts the release flow, that's a one-line
 * addition. For now: explicit button, clear states, opt-in install.
 */
export function UpdaterSection() {
  const [state, setState] = useState<State>({ kind: "idle" });

  const checkForUpdate = async () => {
    setState({ kind: "checking" });
    try {
      const update = await check();
      if (!update) {
        setState({ kind: "no-update", version: "current" });
      } else {
        setState({ kind: "available", update });
      }
    } catch (e) {
      setState({ kind: "error", message: String(e) });
    }
  };

  const installAndRestart = async (update: Update) => {
    setState({ kind: "installing", downloaded: 0, total: null });
    try {
      let total: number | null = null;
      let downloaded = 0;
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            total = event.data.contentLength ?? null;
            setState({ kind: "installing", downloaded: 0, total });
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            setState({ kind: "installing", downloaded, total });
            break;
          case "Finished":
            setState({ kind: "ready-to-restart" });
            break;
        }
      });
      await relaunch();
    } catch (e) {
      setState({ kind: "error", message: String(e) });
    }
  };

  return (
    <div className="space-y-2">
      <Body state={state} />
      <Actions state={state} onCheck={checkForUpdate} onInstall={installAndRestart} />
    </div>
  );
}

function Body({ state }: { state: State }) {
  switch (state.kind) {
    case "idle":
      return (
        <p className="text-[11px] text-muted-foreground">
          Click below to check for a newer version of DevDock.
        </p>
      );
    case "checking":
      return <p className="text-[11px] text-muted-foreground">Checking…</p>;
    case "no-update":
      return (
        <p className="text-[11px] text-muted-foreground">
          You're up to date.
        </p>
      );
    case "available":
      return (
        <div className="space-y-1">
          <p className="text-xs font-medium">
            Version {state.update.version} available
          </p>
          {state.update.body && (
            <pre className="max-h-24 overflow-auto rounded-md border border-border bg-background/60 p-2 font-mono text-[10px] leading-snug whitespace-pre-wrap">
              {state.update.body}
            </pre>
          )}
        </div>
      );
    case "installing": {
      const pct =
        state.total && state.total > 0
          ? Math.min(100, Math.round((state.downloaded / state.total) * 100))
          : null;
      return (
        <div className="space-y-1">
          <p className="text-[11px] text-muted-foreground">
            Downloading update… {pct != null ? `${pct}%` : ""}
          </p>
          <div className="h-1 overflow-hidden rounded-full bg-muted">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: pct != null ? `${pct}%` : "30%" }}
            />
          </div>
        </div>
      );
    }
    case "ready-to-restart":
      return (
        <p className="text-[11px] text-muted-foreground">
          Update installed. DevDock will restart.
        </p>
      );
    case "error":
      return (
        <p className="text-[11px] text-destructive" role="alert">
          {state.message}
        </p>
      );
  }
}

function Actions({
  state,
  onCheck,
  onInstall,
}: {
  state: State;
  onCheck: () => void | Promise<void>;
  onInstall: (update: Update) => void | Promise<void>;
}) {
  switch (state.kind) {
    case "available":
      return (
        <Button
          variant="default"
          size="sm"
          onClick={() => void onInstall(state.update)}
        >
          Install and restart
        </Button>
      );
    case "installing":
    case "ready-to-restart":
      return null;
    default:
      return (
        <Button
          variant="outline"
          size="sm"
          onClick={() => void onCheck()}
          disabled={state.kind === "checking"}
        >
          Check for updates
        </Button>
      );
  }
}
