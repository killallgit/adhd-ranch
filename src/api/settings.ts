import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types/settings";

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function updateSettings(settings: Settings): Promise<void> {
  return invoke("update_settings", { settings });
}
