import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { PenLayout } from "../lib/session/pens";
import type { Animal, FocusAnimal } from "../types/animal";
import type { Rect } from "../types/display";
import type { Pen } from "../types/generated/Pen";
import {
  DRAG_THRESHOLD,
  HITBOX_PADDING,
  PIG_SIZE,
  PIG_SPEED,
  TOSS_VELOCITY_WINDOW_MS,
  buildHitRects,
  computeTossVelocity,
  usePigMovement,
} from "./usePigMovement";
import type { PigState, PointerSample } from "./usePigMovement";

vi.mock("../api/pig", () => ({
  setPigDragActive: vi.fn().mockResolvedValue(undefined),
  subscribeDisplaySpace: vi.fn().mockResolvedValue(() => {}),
  subscribeGatherPigs: vi.fn().mockResolvedValue(() => {}),
  updatePigRects: vi.fn().mockResolvedValue(undefined),
}));

const makePig = (overrides?: Partial<PigState>): PigState => ({
  id: "test",
  name: "Test",
  x: 100,
  y: 200,
  vx: 0,
  vy: 0,
  frameIndex: 0,
  direction: "front",
  lastFrameAt: 0,
  nextTurnAt: 0,
  ...overrides,
});

const focus = (id: string, title: string) => ({
  id,
  title,
  description: "",
  created_at: "",
  tasks: [],
});

const animal = (overrides?: Partial<FocusAnimal>): Animal => {
  const id = overrides?.id ?? "a";
  const name = overrides?.name ?? "Alpha";
  return {
    kind: "focus",
    resting: false,
    scale: 1,
    ...overrides,
    id,
    name,
    focus: focus(id, name),
  };
};

const pen = (id: string): Pen => ({ id, name: id.split("/").pop() ?? id });

const agentAnimal = (sessionId: string, sessionPen: Pen): Animal => ({
  kind: "agent",
  id: `agent:${sessionId}`,
  name: sessionId,
  resting: false,
  scale: 1,
  session: { id: sessionId, name: sessionId, pen: sessionPen, activity: "Working" },
});

function makeSamples(points: { x: number; y: number; t: number }[]): PointerSample[] {
  return points;
}

// Until Rust emits a DisplaySpace the hook measures the document, and jsdom reports a
// zero-sized one — every pen would collapse onto the same empty cell.
function stubRanchViewport(width: number, height: number): () => void {
  const clientWidth = vi
    .spyOn(document.documentElement, "clientWidth", "get")
    .mockReturnValue(width);
  const clientHeight = vi
    .spyOn(document.documentElement, "clientHeight", "get")
    .mockReturnValue(height);
  return () => {
    clientWidth.mockRestore();
    clientHeight.mockRestore();
  };
}

function overlaps(a: Rect, b: Rect): boolean {
  return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
}

function holds(rect: Rect, pig: PigState): boolean {
  return (
    pig.x >= rect.x &&
    pig.y >= rect.y &&
    pig.x + PIG_SIZE <= rect.x + rect.w &&
    pig.y + PIG_SIZE <= rect.y + rect.h
  );
}

function penHolding(pens: readonly PenLayout[], pig: PigState): string | null {
  const home = pens.find((layout) => holds(layout.rect, pig));
  return home ? home.pen.id : null;
}

describe("HITBOX_PADDING", () => {
  it("is exported and equals 16", () => {
    expect(HITBOX_PADDING).toBe(16);
  });
});

describe("buildHitRects", () => {
  it("rect size is PIG_SIZE plus HITBOX_PADDING scaled by dpr", () => {
    const dpr = 2;
    const [rect] = buildHitRects([makePig()], dpr);
    expect(rect?.size).toBe((PIG_SIZE + HITBOX_PADDING) * dpr);
  });

  it("rect x is centered — offset by half padding inward", () => {
    const dpr = 1;
    const pig = makePig({ x: 100 });
    const [rect] = buildHitRects([pig], dpr);
    expect(rect?.x).toBe((100 - HITBOX_PADDING / 2) * dpr);
  });

  it("rect y is centered — offset by half padding inward", () => {
    const dpr = 1;
    const pig = makePig({ y: 200 });
    const [rect] = buildHitRects([pig], dpr);
    expect(rect?.y).toBe((200 - HITBOX_PADDING / 2) * dpr);
  });

  it("dpr scales all dimensions", () => {
    const dpr = 3;
    const pig = makePig({ x: 50, y: 75 });
    const [rect] = buildHitRects([pig], dpr);
    expect(rect?.x).toBe((50 - HITBOX_PADDING / 2) * dpr);
    expect(rect?.y).toBe((75 - HITBOX_PADDING / 2) * dpr);
    expect(rect?.size).toBe((PIG_SIZE + HITBOX_PADDING) * dpr);
  });

  it("returns one rect per pig", () => {
    const pigs = [makePig({ id: "a" }), makePig({ id: "b" }), makePig({ id: "c" })];
    expect(buildHitRects(pigs, 1)).toHaveLength(3);
  });

  it("uses scaled animal bounds when scale is supplied", () => {
    const [rect] = buildHitRects([makePig()], 1, new Map([["test", 3]]));
    expect(rect?.x).toBe(100 - HITBOX_PADDING / 2);
    expect(rect?.y).toBe(200 - HITBOX_PADDING / 2);
    expect(rect?.size).toBe(PIG_SIZE * 3 + HITBOX_PADDING);
  });
});

describe("DRAG_THRESHOLD", () => {
  it("is exported and equals 4", () => {
    expect(DRAG_THRESHOLD).toBe(4);
  });
});

describe("computeTossVelocity", () => {
  it("returns zero velocity for empty samples", () => {
    const result = computeTossVelocity([], TOSS_VELOCITY_WINDOW_MS, 1000);
    expect(result).toEqual({ vx: 0, vy: 0 });
  });

  it("returns zero velocity for a single sample", () => {
    const samples = makeSamples([{ x: 100, y: 100, t: 950 }]);
    const result = computeTossVelocity(samples, TOSS_VELOCITY_WINDOW_MS, 1000);
    expect(result).toEqual({ vx: 0, vy: 0 });
  });

  it("returns zero velocity when all samples are outside the window", () => {
    const samples = makeSamples([
      { x: 0, y: 0, t: 100 },
      { x: 50, y: 0, t: 200 },
    ]);
    const result = computeTossVelocity(samples, TOSS_VELOCITY_WINDOW_MS, 1000);
    expect(result).toEqual({ vx: 0, vy: 0 });
  });

  it("computes velocity from constant-speed samples within window", () => {
    // 5 px in 50 ms = 100 px/s (under the PIG_SPEED*6 cap)
    const now = 1000;
    const samples = makeSamples([
      { x: 0, y: 0, t: now - 50 },
      { x: 5, y: 0, t: now },
    ]);
    const result = computeTossVelocity(samples, TOSS_VELOCITY_WINDOW_MS, now);
    expect(result.vx).toBeCloseTo(100);
    expect(result.vy).toBeCloseTo(0);
  });

  it("ignores samples older than the window", () => {
    const now = 1000;
    // Old sample at x=50 would drag the average down if included.
    // Only the two recent samples should be used: 2px in 40ms = 50 px/s.
    const samples = makeSamples([
      { x: 0, y: 0, t: now - 200 }, // outside 80ms window — ignored
      { x: 0, y: 0, t: now - 40 }, // inside
      { x: 2, y: 0, t: now }, // inside
    ]);
    const result = computeTossVelocity(samples, TOSS_VELOCITY_WINDOW_MS, now);
    expect(result.vx).toBeCloseTo(50);
    expect(result.vy).toBeCloseTo(0);
  });

  it("clamps velocity to PIG_SPEED * 6", () => {
    const now = 1000;
    const samples = makeSamples([
      { x: 0, y: 0, t: now - 10 },
      { x: 10000, y: 0, t: now },
    ]);
    const result = computeTossVelocity(samples, TOSS_VELOCITY_WINDOW_MS, now);
    const maxV = PIG_SPEED * 6;
    expect(result.vx).toBeCloseTo(maxV);
    expect(result.vy).toBeCloseTo(0);
  });

  it("clamps diagonal velocity preserving direction", () => {
    const now = 1000;
    const samples = makeSamples([
      { x: 0, y: 0, t: now - 10 },
      { x: 10000, y: 10000, t: now },
    ]);
    const result = computeTossVelocity(samples, TOSS_VELOCITY_WINDOW_MS, now);
    const speed = Math.sqrt(result.vx ** 2 + result.vy ** 2);
    expect(speed).toBeCloseTo(PIG_SPEED * 6);
    expect(result.vx).toBeCloseTo(result.vy);
  });
});

describe("usePigMovement", () => {
  it("refreshes an existing animal label when its name changes", async () => {
    const rafSpy = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(0);
    const cancelRafSpy = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const original = animal({ name: "Original name" });

    const { result, rerender, unmount } = renderHook(
      ({ animals }: { animals: readonly Animal[] }) => usePigMovement(animals, null),
      { initialProps: { animals: [original] } },
    );

    await waitFor(() => expect(result.current.pigs[0]?.name).toBe("Original name"));
    const firstPosition = {
      x: result.current.pigs[0]?.x,
      y: result.current.pigs[0]?.y,
    };

    rerender({ animals: [{ ...original, name: "Updated name" }] });

    await waitFor(() => expect(result.current.pigs[0]?.name).toBe("Updated name"));
    expect(result.current.pigs[0]).toMatchObject(firstPosition);

    unmount();
    rafSpy.mockRestore();
    cancelRafSpy.mockRestore();
  });

  it("stops resting animals and faces them away", async () => {
    const rafSpy = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(0);
    const cancelRafSpy = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const running = animal();

    const { result, rerender, unmount } = renderHook(
      ({ animals }: { animals: readonly Animal[] }) => usePigMovement(animals, null),
      { initialProps: { animals: [running] } },
    );

    await waitFor(() => expect(result.current.pigs[0]?.id).toBe("a"));
    const firstPosition = {
      x: result.current.pigs[0]?.x,
      y: result.current.pigs[0]?.y,
    };

    rerender({ animals: [{ ...running, resting: true }] });

    await waitFor(() => expect(result.current.pigs[0]?.direction).toBe("back"));
    expect(result.current.pigs[0]).toMatchObject({
      ...firstPosition,
      vx: 0,
      vy: 0,
      direction: "back",
    });

    unmount();
    rafSpy.mockRestore();
    cancelRafSpy.mockRestore();
  });

  it("gives two pens patches of the ranch that do not overlap", async () => {
    const rafSpy = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(0);
    const cancelRafSpy = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const restoreViewport = stubRanchViewport(1200, 800);
    const animals: readonly Animal[] = [
      agentAnimal("session-1", pen("/code/alpha")),
      agentAnimal("session-2", pen("/code/beta")),
    ];

    const { result, unmount } = renderHook(() => usePigMovement(animals, null));

    await waitFor(() => expect(result.current.pens).toHaveLength(2));
    const [first, second] = result.current.pens;
    expect(overlaps(first.rect, second.rect)).toBe(false);

    unmount();
    restoreViewport();
    rafSpy.mockRestore();
    cancelRafSpy.mockRestore();
  });

  it("spawns each agent animal inside the pen of its own session", async () => {
    const rafSpy = vi.spyOn(window, "requestAnimationFrame").mockReturnValue(0);
    const cancelRafSpy = vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => {});
    const randomSpy = vi.spyOn(Math, "random").mockReturnValue(0.5);
    const restoreViewport = stubRanchViewport(1200, 800);
    const animals: readonly Animal[] = [
      agentAnimal("session-1", pen("/code/alpha")),
      agentAnimal("session-2", pen("/code/beta")),
    ];

    const { result, unmount } = renderHook(() => usePigMovement(animals, null));

    await waitFor(() => expect(result.current.pigs).toHaveLength(2));
    const homes = result.current.pigs.map((pig) => penHolding(result.current.pens, pig));
    expect(homes).toEqual(["/code/alpha", "/code/beta"]);

    unmount();
    restoreViewport();
    randomSpy.mockRestore();
    rafSpy.mockRestore();
    cancelRafSpy.mockRestore();
  });
});
