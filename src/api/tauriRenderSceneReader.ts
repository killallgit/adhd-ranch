import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { DisplaySpace } from "../types/display";
import type { RenderRect } from "../types/generated/RenderRect";
import type { RenderScene } from "../types/generated/RenderScene";
import type { Unsubscribe } from "./polledReader";

export interface RenderSceneReader {
  read(area: RenderRect): Promise<RenderScene>;
  subscribe(onChange: (invalidation: RenderSceneInvalidation) => void): Promise<Unsubscribe>;
}

export type RenderSceneInvalidation =
  | { readonly event: "focuses-changed" | "agent-sessions-changed" | "settings-changed" }
  | { readonly event: "display-space"; readonly displaySpace: DisplaySpace };

export function createTauriRenderSceneReader(): RenderSceneReader {
  return {
    read: (area) => invoke<RenderScene>("get_render_scene", { request: { area } }),
    subscribe: async (onChange) => {
      const unlisteners = await Promise.all([
        listen("focuses-changed", () => onChange({ event: "focuses-changed" })),
        listen("agent-sessions-changed", () => onChange({ event: "agent-sessions-changed" })),
        listen("settings-changed", () => onChange({ event: "settings-changed" })),
        listen<DisplaySpace>("display-space", ({ payload }) =>
          onChange({ event: "display-space", displaySpace: payload }),
        ),
      ]);
      return () => {
        for (const unlisten of unlisteners) unlisten();
      };
    },
  };
}
