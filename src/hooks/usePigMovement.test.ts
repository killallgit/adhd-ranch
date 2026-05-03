import { describe, expect, it } from "vitest";
import { HITBOX_PADDING, PIG_SIZE, buildHitRects } from "./usePigMovement";
import type { PigState } from "./usePigMovement";

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
