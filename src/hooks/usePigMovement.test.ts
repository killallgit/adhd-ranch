import { describe, expect, it } from "vitest";
import {
  DRAG_THRESHOLD,
  HITBOX_PADDING,
  PIG_SIZE,
  PIG_SPEED,
  TOSS_VELOCITY_WINDOW_MS,
  buildHitRects,
  computeTossVelocity,
} from "./usePigMovement";
import type { PigState, PointerSample } from "./usePigMovement";

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

function makeSamples(points: { x: number; y: number; t: number }[]): PointerSample[] {
  return points;
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
