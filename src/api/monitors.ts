import { invoke } from "@tauri-apps/api/core";
import type { MonitorInfo } from "../types/monitor";

export async function getMonitors(): Promise<MonitorInfo[]> {
  return invoke<MonitorInfo[]>("get_monitors");
}
