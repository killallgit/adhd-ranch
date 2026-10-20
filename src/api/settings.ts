import { invoke } from "@tauri-apps/api/core";
import type { NotificationSourceDescriptor } from "../types/generated/NotificationSourceDescriptor";
import type { TimerPresetEdit } from "../types/generated/TimerPresetEdit";
import type { TimerPresets } from "../types/generated/TimerPresets";
import type { Settings } from "../types/settings";

export async function listNotificationSources(): Promise<NotificationSourceDescriptor[]> {
  return invoke<NotificationSourceDescriptor[]>("list_notification_sources");
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function updateSettings(settings: Settings): Promise<void> {
  return invoke("update_settings", { settings });
}

export async function editSavedTimerMinutes(edit: TimerPresetEdit): Promise<TimerPresets> {
  return invoke<TimerPresets>("edit_saved_timer_minutes", { edit });
}
