import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { RenderScene } from "../types/generated/RenderScene";
import { createTauriRenderSceneReader } from "./tauriRenderSceneReader";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

const EMPTY_SCENE: RenderScene = { animals: [], regions: [], speciesProfiles: [] };
const DISPLAY_SPACE = {
  span: { w: 1_200, h: 800 },
  spawnRegion: { x: 0, y: 0, w: 1_200, h: 800 },
  movementRegions: [{ x: 0, y: 0, w: 1_200, h: 800 }],
  hitTestScale: 2,
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockListen.mockReset();
});

describe("createTauriRenderSceneReader", () => {
  it("reads get_render_scene with the current area", async () => {
    mockInvoke.mockResolvedValueOnce(EMPTY_SCENE);
    const reader = createTauriRenderSceneReader();
    const area = { x: 10, y: 20, w: 1_200, h: 800 };

    await expect(reader.read(area)).resolves.toEqual(EMPTY_SCENE);
    expect(mockInvoke).toHaveBeenCalledWith("get_render_scene", {
      request: { area },
    });
  });

  it.each(["focuses-changed", "agent-sessions-changed", "settings-changed", "display-space"])(
    "invalidates after %s",
    async (eventName) => {
      const listeners = new Map<string, (event: { payload: unknown }) => void>();
      mockListen.mockImplementation((event, callback) => {
        listeners.set(event, callback as (event: { payload: unknown }) => void);
        return Promise.resolve(() => {});
      });
      const onChange = vi.fn();

      await createTauriRenderSceneReader().subscribe(onChange);
      listeners.get(eventName)?.({ payload: DISPLAY_SPACE });

      expect(onChange).toHaveBeenCalledTimes(1);
    },
  );

  it("unsubscribes all four invalidation listeners", async () => {
    const unlisten = [vi.fn(), vi.fn(), vi.fn(), vi.fn()];
    mockListen.mockImplementation((_event, _callback) =>
      Promise.resolve(unlisten[mockListen.mock.calls.length - 1]),
    );

    const unsubscribe = await createTauriRenderSceneReader().subscribe(() => {});
    unsubscribe();

    expect(mockListen).toHaveBeenCalledTimes(4);
    for (const un of unlisten) expect(un).toHaveBeenCalledTimes(1);
  });
});
