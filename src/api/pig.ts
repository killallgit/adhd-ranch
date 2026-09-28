import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PigHitRect } from "../types/pig";

export type Unsubscribe = () => void;

export async function setPigDragActive(active: boolean): Promise<void> {
  await invoke("set_pig_drag_active", { active });
}

export async function updatePigRects(rects: readonly PigHitRect[]): Promise<void> {
  await invoke("update_pig_rects", { rects });
}

export async function subscribeGatherPigs(cb: () => void): Promise<Unsubscribe> {
  return listen("gather-pigs", () => cb());
}

export async function subscribeOpenFocusDetail(
  cb: (focusId: string) => void,
): Promise<Unsubscribe> {
  return listen<string>("open-focus-detail", (event) => cb(event.payload));
}
