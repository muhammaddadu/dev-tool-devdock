import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Handlers = {
  onRefresh?: () => void | Promise<void>;
  onFocusSearch?: () => void;
};

/**
 * Register the menu-bar shortcuts. Esc hides the panel. Cmd-R triggers a
 * refresh. Cmd-F focuses the search input. Mounted once at the panel root.
 */
export function useKeyboardShortcuts({ onRefresh, onFocusSearch }: Handlers) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Esc — hide panel (menu-bar app convention).
      if (e.key === "Escape" && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        void getCurrentWindow().hide();
        return;
      }
      // Cmd-R — refresh. We swallow the event so the webview doesn't reload.
      if (e.key.toLowerCase() === "r" && e.metaKey && !e.shiftKey) {
        e.preventDefault();
        void onRefresh?.();
        return;
      }
      // Cmd-F — focus the search input.
      if (e.key.toLowerCase() === "f" && e.metaKey && !e.shiftKey) {
        e.preventDefault();
        onFocusSearch?.();
        return;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onRefresh, onFocusSearch]);
}
