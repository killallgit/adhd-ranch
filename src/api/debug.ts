import { invoke } from "@tauri-apps/api/core";

export async function getDebugOverlay(): Promise<boolean> {
  return invoke<boolean>("get_debug_overlay");
}

export async function setDebugOverlay(enabled: boolean): Promise<void> {
  return invoke("set_debug_overlay", { enabled });
}

export async function getDevtoolsOpen(): Promise<boolean> {
  return invoke<boolean>("get_devtools_open");
}

export async function toggleDevtools(): Promise<void> {
  return invoke("toggle_devtools");
}
