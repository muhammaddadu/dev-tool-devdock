import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import * as api from "@/lib/api";
import type { ServiceView } from "@/lib/types";

type Props = {
  service: ServiceView;
  open: boolean;
  onClose: () => void;
};

const POLL_MS = 1000;
const TAIL_BYTES = 64 * 1024;

/**
 * Live tail of stdout/stderr for a managed service. While open, polls the log
 * file once per second. Auto-scrolls to the bottom unless the user has
 * scrolled up to read — then it sticks where they are.
 */
export function LogsDialog({ service, open, onClose }: Props) {
  const [content, setContent] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [stuckToBottom, setStuckToBottom] = useState(true);
  const preRef = useRef<HTMLPreElement>(null);

  const path = service.logPath;

  useEffect(() => {
    if (!open || !path) return;

    let cancelled = false;
    const fetchOnce = async () => {
      try {
        const text = await api.readLogTail(path, TAIL_BYTES);
        if (!cancelled) {
          setContent(text);
          setError(null);
        }
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    };

    void fetchOnce();
    const id = setInterval(() => void fetchOnce(), POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [open, path]);

  useEffect(() => {
    if (!stuckToBottom) return;
    const el = preRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [content, stuckToBottom]);

  const onScroll = () => {
    const el = preRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 8;
    setStuckToBottom(atBottom);
  };

  if (!path) return null;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={`Logs · ${service.label}`}
      className="max-w-xl"
    >
      <div className="space-y-3">
        <div className="flex items-center justify-between gap-2">
          <p className="truncate font-mono text-[10px] text-muted-foreground" title={path}>
            {path}
          </p>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => void api.openPath(path).catch(() => {})}
          >
            Open file
          </Button>
        </div>
        <pre
          ref={preRef}
          onScroll={onScroll}
          className="h-80 overflow-auto rounded-md border border-border bg-background/60 p-2 font-mono text-[11px] leading-snug whitespace-pre-wrap break-words"
        >
          {error
            ? `Couldn't read log: ${error}`
            : content || "Waiting for output…"}
        </pre>
        <div className="flex items-center justify-between">
          <span className="text-[10px] text-muted-foreground">
            {stuckToBottom ? "Auto-scrolling" : "Paused — scroll to bottom to resume"}
          </span>
          <Button variant="outline" size="md" onClick={onClose}>
            Close
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
