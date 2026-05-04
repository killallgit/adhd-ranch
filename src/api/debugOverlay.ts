import { listen } from "@tauri-apps/api/event";

export type DebugOverlayListener = (visible: boolean) => void;
export type Unsubscribe = () => void;

export async function subscribeDebugOverlay(onToggle: DebugOverlayListener): Promise<Unsubscribe> {
  const unlisten = await listen<boolean>("debug-overlay-toggle", (e) => {
    onToggle(e.payload);
  });
  return unlisten;
}
