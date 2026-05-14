import { create } from "zustand";
import type { EditorInfo, ServiceView } from "./types";
import * as api from "./api";

export const DEFAULT_REFRESH_INTERVAL_MS = 1500;

type AppStore = {
  services: ServiceView[];
  editors: EditorInfo[];
  selectedServiceId: string | null;
  searchQuery: string;
  isRefreshing: boolean;
  lastError: string | null;
  refreshIntervalMs: number;

  setSearchQuery: (q: string) => void;
  selectService: (id: string | null) => void;
  refresh: () => Promise<void>;
  loadEditors: () => Promise<void>;
  loadSettings: () => Promise<void>;
  setRefreshIntervalMs: (value: number) => Promise<void>;
  runService: (id: string) => Promise<void>;
  stopService: (id: string) => Promise<void>;
  restartService: (id: string) => Promise<void>;
};

export const useAppStore = create<AppStore>((set, get) => ({
  services: [],
  editors: [],
  selectedServiceId: null,
  searchQuery: "",
  isRefreshing: false,
  lastError: null,
  refreshIntervalMs: DEFAULT_REFRESH_INTERVAL_MS,

  setSearchQuery: (q) => set({ searchQuery: q }),
  selectService: (id) => set({ selectedServiceId: id }),

  loadEditors: async () => {
    try {
      const editors = await api.listEditors();
      set({ editors });
    } catch {
      // Non-fatal — the editor menu just won't show entries.
    }
  },

  loadSettings: async () => {
    try {
      const raw = await api.listSettings();
      const interval = parseInt(raw.refresh_interval_ms ?? "", 10);
      if (Number.isFinite(interval) && interval >= 500 && interval <= 60_000) {
        set({ refreshIntervalMs: interval });
      }
    } catch {
      // Stay on defaults.
    }
  },

  setRefreshIntervalMs: async (value) => {
    const clamped = Math.min(60_000, Math.max(500, Math.round(value)));
    set({ refreshIntervalMs: clamped });
    try {
      await api.setSetting("refresh_interval_ms", String(clamped));
    } catch {
      // Best-effort. The in-memory store still reflects the user's choice.
    }
  },

  refresh: async () => {
    set({ isRefreshing: true, lastError: null });
    try {
      const services = await api.refreshServices();
      set({ services, isRefreshing: false });
    } catch (e) {
      set({ isRefreshing: false, lastError: String(e) });
    }
  },

  runService: async (id) => {
    try {
      await api.runService(id);
      await get().refresh();
    } catch (e) {
      set({ lastError: String(e) });
    }
  },

  stopService: async (id) => {
    try {
      await api.stopService(id);
      await get().refresh();
    } catch (e) {
      set({ lastError: String(e) });
    }
  },

  restartService: async (id) => {
    try {
      await api.restartService(id);
      await get().refresh();
    } catch (e) {
      set({ lastError: String(e) });
    }
  },
}));
