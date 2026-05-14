import { useEffect, useState } from "react";
import { useTheme } from "next-themes";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/cn";
import { useAppStore } from "@/lib/store";
import { UpdaterSection } from "./UpdaterSection";

type Props = {
  open: boolean;
  onClose: () => void;
};

const THEME_OPTIONS = [
  { value: "system", label: "System" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
] as const;

export function SettingsDialog({ open, onClose }: Props) {
  const refreshIntervalMs = useAppStore((s) => s.refreshIntervalMs);
  const setRefreshIntervalMs = useAppStore((s) => s.setRefreshIntervalMs);
  const { theme, setTheme } = useTheme();

  const [intervalDraft, setIntervalDraft] = useState(String(refreshIntervalMs));
  const [autostartOn, setAutostartOn] = useState<boolean | null>(null);
  const [autostartError, setAutostartError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setIntervalDraft(String(refreshIntervalMs));
    void isEnabled()
      .then(setAutostartOn)
      .catch((e) => setAutostartError(String(e)));
  }, [open, refreshIntervalMs]);

  const toggleAutostart = async () => {
    setAutostartError(null);
    try {
      if (autostartOn) {
        await disable();
        setAutostartOn(false);
      } else {
        await enable();
        setAutostartOn(true);
      }
    } catch (e) {
      setAutostartError(String(e));
    }
  };

  const commitInterval = async () => {
    const parsed = parseInt(intervalDraft, 10);
    if (!Number.isFinite(parsed)) return;
    await setRefreshIntervalMs(parsed);
    setIntervalDraft(String(parsed));
  };

  return (
    <Dialog open={open} onClose={onClose} title="Settings" className="max-w-md">
      <div className="space-y-5">
        <Section title="Behavior">
          <Row
            label="Start at login"
            hint="DevDock launches automatically when you log in."
          >
            <ToggleButton
              on={!!autostartOn}
              onClick={() => void toggleAutostart()}
              disabled={autostartOn === null}
            />
          </Row>
          <Row
            label="Refresh interval"
            hint="How often DevDock checks listening ports while the panel is open."
          >
            <div className="flex items-center gap-2">
              <Input
                type="number"
                min={500}
                max={60_000}
                step={100}
                value={intervalDraft}
                onChange={(e) => setIntervalDraft(e.target.value)}
                onBlur={() => void commitInterval()}
                className="w-24 text-right tabular-nums"
              />
              <span className="text-[11px] text-muted-foreground">ms</span>
            </div>
          </Row>
        </Section>

        <Section title="Appearance">
          <Row label="Theme">
            <div className="flex gap-1 rounded-md border border-border bg-background p-0.5">
              {THEME_OPTIONS.map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  onClick={() => setTheme(opt.value)}
                  className={cn(
                    "rounded px-2.5 py-1 text-xs",
                    (theme ?? "system") === opt.value
                      ? "bg-primary text-primary-foreground"
                      : "text-muted-foreground hover:text-foreground",
                  )}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </Row>
        </Section>

        <Section title="Updates">
          <UpdaterSection />
        </Section>

        {autostartError && (
          <p className="text-[11px] text-destructive" role="alert">
            {autostartError}
          </p>
        )}

        <div className="flex justify-end gap-2 pt-1">
          <Button variant="default" size="md" onClick={onClose}>
            Done
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-2">
      <h3 className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        {title}
      </h3>
      <div className="space-y-3 rounded-md border border-border bg-background/40 px-3 py-2.5">
        {children}
      </div>
    </div>
  );
}

function Row({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="min-w-0 flex-1">
        <p className="text-xs font-medium">{label}</p>
        {hint && <p className="text-[11px] text-muted-foreground">{hint}</p>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function ToggleButton({
  on,
  onClick,
  disabled,
}: {
  on: boolean;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "relative inline-flex h-5 w-9 items-center rounded-full border border-border transition-colors",
        on ? "bg-primary" : "bg-muted",
        disabled && "cursor-not-allowed opacity-50",
      )}
    >
      <span
        className={cn(
          "inline-block h-3.5 w-3.5 rounded-full bg-background shadow-sm transition-transform",
          on ? "translate-x-[18px]" : "translate-x-0.5",
        )}
      />
    </button>
  );
}
