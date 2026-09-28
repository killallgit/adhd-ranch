import { invoke } from "@tauri-apps/api/core";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DisplaySpace } from "../types/display";
import type { RenderScene } from "../types/generated/RenderScene";
import { buildHitRects, useAnimalMovement } from "./useAnimalMovement";

const pigApi = vi.hoisted(() => ({
  gather: null as null | (() => void),
  setPigDragActive: vi.fn().mockResolvedValue(undefined),
  updatePigRects: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../api/pig", () => ({
  setPigDragActive: pigApi.setPigDragActive,
  subscribeGatherPigs: vi.fn().mockImplementation(async (callback: () => void) => {
    pigApi.gather = callback;
    return () => {};
  }),
  updatePigRects: pigApi.updatePigRects,
}));

const DISPLAY: DisplaySpace = {
  span: { w: 800, h: 600 },
  spawnRegion: { x: 0, y: 0, w: 800, h: 600 },
  movementRegions: [{ x: 0, y: 0, w: 800, h: 600 }],
  hitTestScale: 2,
};

const SCENE: RenderScene = {
  animals: [
    {
      id: "focus-1",
      label: "Focus",
      species: "pig",
      size: { kind: "fixed", px: 48 },
      regionId: null,
      motion: "walking",
    },
    {
      id: "agent:session-1",
      label: "Agent",
      species: "pig",
      size: { kind: "fixed", px: 48 },
      regionId: null,
      motion: "resting",
    },
  ],
  regions: [],
  speciesProfiles: [
    {
      species: "pig",
      baseSizePx: 48,
      movementFootprintPx: 48,
      walkSpeedPxPerSecond: 60,
      friction: 0.97,
      minimumSpeedFraction: 0.35,
      maximumTossSpeedMultiplier: 6,
      turnMinMs: 3_000,
      turnMaxMs: 8_000,
      frameIntervalMs: 150,
    },
  ],
};

beforeEach(() => {
  pigApi.gather = null;
  pigApi.setPigDragActive.mockClear();
  pigApi.updatePigRects.mockClear();
  vi.mocked(invoke).mockReset();
});

function mockMovementClock() {
  let frame: FrameRequestCallback | undefined;
  let synchronize: TimerHandler | undefined;
  const raf = vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
    frame = callback;
    return 1;
  });
  const cancel = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
  const interval = vi.spyOn(window, "setInterval").mockImplementation((handler, timeout) => {
    if (timeout === 64) synchronize = handler;
    return 1 as unknown as ReturnType<typeof window.setInterval>;
  });
  const clearInterval = vi.spyOn(window, "clearInterval").mockImplementation(() => {});

  return {
    runFrame: (now: number) => frame?.(now),
    synchronize: () => {
      if (typeof synchronize === "function") synchronize();
    },
    clearInterval,
    restore: () => {
      clearInterval.mockRestore();
      interval.mockRestore();
      cancel.mockRestore();
      raf.mockRestore();
    },
  };
}

describe("useAnimalMovement", () => {
  it("keeps one requestAnimationFrame loop for a mixed Focus and Agent roster", async () => {
    const raf = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(1);
    const cancel = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const { result, rerender, unmount } = renderHook(
      ({ scene }: { scene: RenderScene }) => useAnimalMovement(scene, DISPLAY, null),
      { initialProps: { scene: SCENE } },
    );

    await waitFor(() => expect(result.current.animals).toHaveLength(2));
    const initialPosition = {
      x: result.current.animals[0]?.x,
      y: result.current.animals[0]?.y,
    };
    expect(result.current.animals.map((animal) => animal.id)).toEqual([
      "focus-1",
      "agent:session-1",
    ]);

    rerender({
      scene: {
        ...SCENE,
        animals: SCENE.animals.map((animal) =>
          animal.id === "focus-1"
            ? { ...animal, label: "Updated", size: { kind: "fixed" as const, px: 96 } }
            : animal,
        ),
      },
    });
    await waitFor(() => expect(result.current.animals[0]?.label).toBe("Updated"));
    expect(result.current.animals[0]).toMatchObject(initialPosition);
    expect(raf).toHaveBeenCalledTimes(1);

    unmount();
    cancel.mockRestore();
    raf.mockRestore();
  });

  it("keeps backend synchronization out of animation frames", async () => {
    const clock = mockMovementClock();
    const { result, unmount } = renderHook(() => useAnimalMovement(SCENE, DISPLAY, null));
    await waitFor(() => expect(result.current.animals).toHaveLength(2));
    pigApi.updatePigRects.mockClear();

    act(() => clock.runFrame(performance.now() + 16));
    expect(pigApi.updatePigRects).not.toHaveBeenCalled();
    expect(pigApi.setPigDragActive).not.toHaveBeenCalled();

    unmount();
    clock.restore();
  });

  it("synchronizes hit rectangles only at explicit drag boundaries", async () => {
    const clock = mockMovementClock();
    const { result, unmount } = renderHook(() => useAnimalMovement(SCENE, DISPLAY, null));
    await waitFor(() => expect(result.current.animals).toHaveLength(2));
    pigApi.updatePigRects.mockClear();

    let dragOutcome = { wasDrag: false };
    act(() => {
      result.current.setDragActive(true);
      result.current.startDrag("focus-1", 100, 100);
      result.current.moveDrag(160, 100);
      dragOutcome = result.current.endDrag();
      result.current.setDragActive(false);
    });
    expect(dragOutcome.wasDrag).toBe(true);
    expect(pigApi.updatePigRects).toHaveBeenCalledTimes(2);
    expect(pigApi.updatePigRects.mock.calls[0]?.[0]).toEqual([{ x: 0, y: 0, size: 3_200 }]);
    expect(pigApi.updatePigRects.mock.calls[1]?.[0]).toHaveLength(2);
    expect(pigApi.setPigDragActive.mock.calls).toEqual([[true], [false]]);

    act(() => clock.runFrame(performance.now() + 16));
    expect(pigApi.updatePigRects).toHaveBeenCalledTimes(2);
    expect(pigApi.setPigDragActive).toHaveBeenCalledTimes(2);

    unmount();
    clock.restore();
  });

  it("widens the selected hit region and narrows it after close", async () => {
    const clock = mockMovementClock();
    const { result, rerender, unmount } = renderHook(
      ({ frozenId }: { frozenId: string | null }) => useAnimalMovement(SCENE, DISPLAY, frozenId),
      { initialProps: { frozenId: null as string | null } },
    );
    await waitFor(() => expect(result.current.animals).toHaveLength(2));
    pigApi.updatePigRects.mockClear();

    rerender({ frozenId: "focus-1" });
    act(clock.synchronize);
    expect(pigApi.updatePigRects.mock.calls[0]?.[0]).toEqual([{ x: 0, y: 0, size: 3_200 }]);

    rerender({ frozenId: null });
    act(clock.synchronize);
    expect(pigApi.updatePigRects.mock.calls[1]?.[0]).toHaveLength(2);

    unmount();
    clock.restore();
  });

  it("clears the hit rectangle interval on unmount", () => {
    const clock = mockMovementClock();
    const { unmount } = renderHook(() => useAnimalMovement(SCENE, DISPLAY, null));

    unmount();
    expect(clock.clearInterval).toHaveBeenCalledTimes(1);
    clock.restore();
  });

  it("keeps a selected Focus fixed when its Motion becomes resting", async () => {
    const raf = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(1);
    const cancel = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const random = vi.spyOn(Math, "random").mockReturnValue(0.9);
    const compactDisplay: DisplaySpace = {
      span: { w: 100, h: 100 },
      spawnRegion: { x: 0, y: 0, w: 100, h: 100 },
      movementRegions: [{ x: 0, y: 0, w: 100, h: 100 }],
      hitTestScale: 2,
    };
    const { result, rerender, unmount } = renderHook(
      ({ scene, display }: { scene: RenderScene; display: DisplaySpace }) =>
        useAnimalMovement(scene, display, "focus-1"),
      { initialProps: { scene: SCENE, display: DISPLAY } },
    );
    await waitFor(() => expect(result.current.animals).toHaveLength(2));
    const before = result.current.animals.find((animal) => animal.id === "focus-1");

    rerender({
      scene: {
        ...SCENE,
        animals: SCENE.animals.map((animal) =>
          animal.id === "focus-1" ? { ...animal, motion: "resting" as const } : animal,
        ),
      },
      display: compactDisplay,
    });

    await waitFor(() => {
      expect(result.current.animals.find((animal) => animal.id === "focus-1")).toMatchObject({
        x: before?.x,
        y: before?.y,
      });
    });

    unmount();
    random.mockRestore();
    cancel.mockRestore();
    raf.mockRestore();
  });

  it("builds hit rectangles from sampled absolute-pixel size", () => {
    const rects = buildHitRects(
      [
        {
          id: "focus-1",
          label: "Focus",
          x: 100,
          y: 200,
          vx: 0,
          vy: 0,
          frameIndex: 0,
          direction: "front",
          lastFrameAt: 0,
          nextTurnAt: 0,
        },
      ],
      2,
      new Map([["focus-1", 96]]),
    );

    expect(rects[0]).toMatchObject({ x: 184, y: 384, size: 224 });
  });

  it("gathers animals inside their assigned regions", async () => {
    const raf = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(1);
    const cancel = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const assignedRect = { x: 300, y: 200, w: 160, h: 160 };
    const regionalScene: RenderScene = {
      ...SCENE,
      animals: SCENE.animals.map((animal) =>
        animal.id === "agent:session-1" ? { ...animal, regionId: "repo" } : animal,
      ),
      regions: [{ id: "repo", label: "Repository", rect: assignedRect, hue: 12 }],
    };
    const { result, unmount } = renderHook(() => useAnimalMovement(regionalScene, DISPLAY, null));
    await waitFor(() => expect(result.current.animals).toHaveLength(2));

    act(() => pigApi.gather?.());
    const gatheredAgent = result.current.animals.find((animal) => animal.id === "agent:session-1");
    expect(gatheredAgent?.x).toBeGreaterThanOrEqual(assignedRect.x);
    expect(gatheredAgent?.x).toBeLessThanOrEqual(assignedRect.x + assignedRect.w - 48);
    expect(gatheredAgent?.y).toBeGreaterThanOrEqual(assignedRect.y);
    expect(gatheredAgent?.y).toBeLessThanOrEqual(assignedRect.y + assignedRect.h - 48);

    unmount();
    cancel.mockRestore();
    raf.mockRestore();
  });

  it("supports drag/toss without reading the render scene", async () => {
    const raf = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(1);
    const cancel = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const random = vi.spyOn(Math, "random").mockReturnValue(0.5);
    const { result, unmount } = renderHook(() => useAnimalMovement(SCENE, DISPLAY, null));
    await waitFor(() => expect(result.current.animals).toHaveLength(2));

    act(() => result.current.startDrag("focus-1", 100, 100));
    act(() => result.current.moveDrag(140, 100));
    let outcome = { wasDrag: false };
    act(() => {
      outcome = result.current.endDrag();
    });
    expect(outcome.wasDrag).toBe(true);
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "get_render_scene")).toBe(
      false,
    );

    unmount();
    random.mockRestore();
    cancel.mockRestore();
    raf.mockRestore();
  });

  it("handles delayed scene/layout data and preserves movement identity across regions", async () => {
    const raf = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(1);
    const cancel = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const empty: RenderScene = { animals: [], regions: [], speciesProfiles: SCENE.speciesProfiles };
    const { result, rerender, unmount } = renderHook(
      ({ scene }: { scene: RenderScene }) => useAnimalMovement(scene, DISPLAY, null),
      { initialProps: { scene: empty } },
    );
    expect(result.current.animals).toEqual([]);

    const unresolved: RenderScene = {
      ...empty,
      animals: [{ ...SCENE.animals[1], regionId: "repo", motion: "walking" }],
    };
    rerender({ scene: unresolved });
    await waitFor(() => expect(result.current.animals).toHaveLength(1));
    const before = result.current.animals[0];

    rerender({
      scene: {
        ...unresolved,
        regions: [
          {
            id: "repo",
            label: "Repository",
            rect: DISPLAY.spawnRegion,
            hue: 12,
          },
        ],
      },
    });
    await waitFor(() => expect(result.current.animals[0]?.id).toBe("agent:session-1"));
    expect(result.current.animals[0]).toMatchObject({ x: before?.x, y: before?.y });
    expect(raf).toHaveBeenCalledTimes(1);

    unmount();
    cancel.mockRestore();
    raf.mockRestore();
  });
});
