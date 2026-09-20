import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types/settings";
import type { PolledReader } from "./polledReader";
import { createTauriReader } from "./tauriReader";

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function updateSettings(settings: Settings): Promise<void> {
  return invoke("update_settings", { settings });
}

// The overlay draws from settings but has no way to know they changed; without the
// event a resized pen would only take effect on the next launch.
export function createSettingsReader(): PolledReader<Settings> {
  return createTauriReader<Settings, Settings>({
    invokeKey: "get_settings",
    eventKey: "settings-changed",
    map: (raw) => raw,
  });
}
