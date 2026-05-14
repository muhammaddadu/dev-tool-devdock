// Shared frontend types. Mirror what the Rust backend returns in serde-serialized form.

export type ServiceStatus =
  | "running"
  | "starting"
  | "stopped"
  | "crashed"
  | "unknown";

export type ServiceSource = "detected" | "managed";

export type ServiceBucket = "dev" | "tooling" | "system";

export type ServiceView = {
  id: string;
  label: string;
  status: ServiceStatus;
  source: ServiceSource;
  bucket: ServiceBucket;
  port: number | null;
  host: string | null;
  url: string | null;
  pid: number | null;
  processName: string | null;
  command: string | null;
  cwd: string | null;
  projectName: string | null;
  projectRoot: string | null;
  startedAtUnix: number | null;
  savedId: string | null;
  logPath: string | null;
  pinned: boolean;
  canOpen: boolean;
  canSave: boolean;
  canRun: boolean;
  canStop: boolean;
  canRestart: boolean;
  canKill: boolean;
};

export type SaveDetectedServiceInput = {
  label: string;
  command: string;
  cwd: string;
  expectedPorts: number[];
  detectedPid: number | null;
};

export type SavedService = {
  id: string;
  projectId: string | null;
  label: string;
  command: string;
  cwd: string;
  expectedPorts: number[];
  pinned: boolean;
  createdFrom: string;
  lastRunAt: string | null;
  lastSeenAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ProjectScanSuggestion = {
  label: string;
  command: string;
  cwd: string;
  expectedPorts: number[];
  confidence: "low" | "medium" | "high";
  reason: string;
};

export type ProjectScanResult = {
  projectRoot: string;
  suggestions: ProjectScanSuggestion[];
};

export type AiProviderId = "claude" | "codex" | "cursor" | "ollama";

export type AiProviderView = {
  id: AiProviderId;
  displayName: string;
  available: boolean;
  version: string | null;
};

export type EditorInfo = {
  id: string;
  displayName: string;
  cliPath: string | null;
  bundleName: string | null;
};
