import { invoke } from "@tauri-apps/api/core";
import type {
  AiProviderView,
  EditorInfo,
  ProjectScanResult,
  SaveDetectedServiceInput,
  SavedService,
  ServiceView,
} from "./types";

export async function listServices(): Promise<ServiceView[]> {
  return invoke<ServiceView[]>("list_services");
}

export async function refreshServices(): Promise<ServiceView[]> {
  return invoke<ServiceView[]>("refresh_services");
}

export async function saveDetectedService(
  input: SaveDetectedServiceInput,
): Promise<SavedService> {
  return invoke<SavedService>("save_detected_service", { input });
}

export type UpdateServiceInput = {
  label: string;
  command: string;
  cwd: string;
  expectedPorts: number[];
};

export async function updateSavedService(
  id: string,
  input: UpdateServiceInput,
): Promise<SavedService> {
  return invoke<SavedService>("update_saved_service", { id, input });
}

export async function unsaveService(id: string): Promise<void> {
  return invoke<void>("unsave_service", { id });
}

export async function setServicePinned(id: string, pinned: boolean): Promise<void> {
  return invoke<void>("set_service_pinned", { id, pinned });
}

export async function runService(serviceId: string): Promise<ServiceView> {
  return invoke<ServiceView>("run_service", { serviceId });
}

export async function stopService(serviceId: string): Promise<void> {
  return invoke<void>("stop_service", { serviceId });
}

export async function restartService(serviceId: string): Promise<ServiceView> {
  return invoke<ServiceView>("restart_service", { serviceId });
}

export async function killDetectedProcess(pid: number): Promise<void> {
  return invoke<void>("kill_detected_process", { pid });
}

export async function openUrl(url: string): Promise<void> {
  return invoke<void>("open_url", { url });
}

export async function openPath(path: string): Promise<void> {
  return invoke<void>("open_path", { path });
}

export async function openInTerminal(path: string): Promise<void> {
  return invoke<void>("open_in_terminal", { path });
}

export async function listEditors(): Promise<EditorInfo[]> {
  return invoke<EditorInfo[]>("list_editors");
}

export async function openInEditor(editor: string, path: string): Promise<void> {
  return invoke<void>("open_in_editor", { editor, path });
}

export async function readLogTail(path: string, maxBytes?: number): Promise<string> {
  return invoke<string>("read_log_tail", { path, maxBytes });
}

export async function listSettings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>("list_settings");
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_setting", { key, value });
}

export async function quitApp(stopManaged: boolean): Promise<void> {
  return invoke<void>("quit_app", { stopManaged });
}

/** Clipboard is handled in the webview — no backend round-trip needed. */
export async function copyToClipboard(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}

export async function scanProject(path: string): Promise<ProjectScanResult> {
  return invoke<ProjectScanResult>("scan_project", { path });
}

export async function detectAiProviders(): Promise<AiProviderView[]> {
  return invoke<AiProviderView[]>("detect_ai_providers");
}
