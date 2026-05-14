import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Refresh the service list on a tick. Two cadences:
 *
 * * `activeIntervalMs` — while the panel is focused.
 * * `idleIntervalMs`   — while the panel is hidden / unfocused.
 *
 * Keeping a slow background tick means the cache is already warm when the
 * user clicks the tray icon, so the panel paints instantly instead of
 * showing a loading state. The idle interval is conservative on purpose;
 * we don't want to shell out to `lsof` aggressively when no one is looking.
 */
export function useAutoRefresh(
  refresh: () => void | Promise<void>,
  activeIntervalMs = 1500,
  idleIntervalMs = 10_000,
) {
  useEffect(() => {
    const win = getCurrentWindow();
    let timerId: ReturnType<typeof setInterval> | null = null;
    let cancelled = false;

    const restart = (intervalMs: number) => {
      if (timerId != null) clearInterval(timerId);
      void refresh();
      timerId = setInterval(() => void refresh(), intervalMs);
    };

    // Kick off immediately so the very first refresh fires at launch,
    // before the user has had a chance to click the tray icon.
    void win
      .isVisible()
      .then((visible) => {
        if (cancelled) return;
        restart(visible ? activeIntervalMs : idleIntervalMs);
      })
      .catch(() => {
        if (!cancelled) restart(idleIntervalMs);
      });

    const unlistenPromise = win.onFocusChanged(({ payload: focused }) => {
      restart(focused ? activeIntervalMs : idleIntervalMs);
    });

    return () => {
      cancelled = true;
      if (timerId != null) clearInterval(timerId);
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refresh, activeIntervalMs, idleIntervalMs]);
}
