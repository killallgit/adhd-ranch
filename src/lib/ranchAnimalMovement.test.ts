import { describe, expect, it } from "vitest";
import type { DisplaySpace } from "../types/display";
import { type RanchAnimalState, advanceRanchAnimal } from "./ranchAnimalMovement";

const displaySpace: DisplaySpace = {
  span: { w: 3200, h: 1080 },
  spawnRegion: { x: 1280, y: 0, w: 1920, h: 1080 },
  movementRegions: [
    { x: 1280, y: 0, w: 1920, h: 1080 },
    { x: 0, y: 0, w: 1280, h: 800 },
  ],
  hitTestScale: 2,
};

const animal = (overrides?: Partial<RanchAnimalState>): RanchAnimalState => ({
  id: "a",
  name: "Alpha",
  x: 1300,
  y: 100,
  vx: 60,
  vy: 0,
  frameIndex: 0,
  direction: "right",
  lastFrameAt: 0,
  nextTurnAt: 3000,
  ...overrides,
});

describe("advanceRanchAnimal", () => {
  it("clamps an invalid position to the nearest movement region", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 1000, y: 900, vx: 0, vy: 0 }),
      displaySpace,
      dtMs: 0,
      nowMs: 0,
      frozen: false,
      random: () => 0,
    });

    expect(next.x).toBe(1000);
    expect(next.y).toBe(800 - 48);
  });

  it("hard-clamps and reflects velocity after crossing a movement-region edge", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 80, y: 40, vx: 120, vy: 0, nextTurnAt: 3000 }),
      displaySpace: {
        span: { w: 200, h: 200 },
        spawnRegion: { x: 0, y: 0, w: 200, h: 200 },
        movementRegions: [{ x: 0, y: 0, w: 200, h: 200 }],
        hitTestScale: 1,
      },
      dtMs: 1000,
      nowMs: 1000,
      frozen: false,
      random: () => 0,
    });

    expect(next.x).toBe(200 - 48);
    expect(next.vx).toBeLessThan(0);
  });

  it("soft-steers away from a movement-region edge before impact", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 120, y: 40, vx: 60, vy: 0, nextTurnAt: 3000 }),
      displaySpace: {
        span: { w: 200, h: 200 },
        spawnRegion: { x: 0, y: 0, w: 200, h: 200 },
        movementRegions: [{ x: 0, y: 0, w: 200, h: 200 }],
        hitTestScale: 1,
      },
      dtMs: 0,
      nowMs: 1000,
      frozen: false,
      random: () => 0,
    });

    expect(next.vx).toBeLessThan(0);
  });

  it("uses the randomness adapter for turn direction and next turn timing", () => {
    const values = [0, 0];
    const next = advanceRanchAnimal({
      animal: animal({ x: 80, y: 80, vx: 0, vy: 0, nextTurnAt: 1000 }),
      displaySpace: {
        span: { w: 400, h: 400 },
        spawnRegion: { x: 0, y: 0, w: 400, h: 400 },
        movementRegions: [{ x: 0, y: 0, w: 400, h: 400 }],
        hitTestScale: 1,
      },
      dtMs: 0,
      nowMs: 1000,
      frozen: false,
      random: () => values.shift() ?? 0,
    });

    expect(next.vx).toBeGreaterThan(0);
    expect(next.vy).toBeCloseTo(0);
    expect(next.nextTurnAt).toBe(4000);
  });

  it("does not move while frozen but advances movement timers", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 80, y: 80, vx: 60, vy: 0, lastFrameAt: 100, nextTurnAt: 3000 }),
      displaySpace,
      dtMs: 250,
      nowMs: 1000,
      frozen: true,
      random: () => 0,
    });

    expect(next.x).toBe(80);
    expect(next.y).toBe(80);
    expect(next.lastFrameAt).toBe(350);
    expect(next.nextTurnAt).toBe(3250);
  });

  it("applies friction to moving animals", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 100, y: 100, vx: 120, vy: 0, nextTurnAt: 3000 }),
      displaySpace: {
        span: { w: 600, h: 600 },
        spawnRegion: { x: 0, y: 0, w: 600, h: 600 },
        movementRegions: [{ x: 0, y: 0, w: 600, h: 600 }],
        hitTestScale: 1,
      },
      dtMs: 0,
      nowMs: 1000,
      frozen: false,
      random: () => 0,
    });

    expect(next.vx).toBeCloseTo(116.4);
  });

  it("keeps a minimum roaming speed after friction", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 100, y: 100, vx: 1, vy: 0, nextTurnAt: 3000 }),
      displaySpace: {
        span: { w: 600, h: 600 },
        spawnRegion: { x: 0, y: 0, w: 600, h: 600 },
        movementRegions: [{ x: 0, y: 0, w: 600, h: 600 }],
        hitTestScale: 1,
      },
      dtMs: 0,
      nowMs: 1000,
      frozen: false,
      random: () => 0,
    });

    expect(Math.hypot(next.vx, next.vy)).toBeCloseTo(21);
  });

  it("advances animation frames on the frame interval", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 100, y: 100, frameIndex: 3, lastFrameAt: 100, nextTurnAt: 3000 }),
      displaySpace: {
        span: { w: 600, h: 600 },
        spawnRegion: { x: 0, y: 0, w: 600, h: 600 },
        movementRegions: [{ x: 0, y: 0, w: 600, h: 600 }],
        hitTestScale: 1,
      },
      dtMs: 0,
      nowMs: 250,
      frozen: false,
      random: () => 0,
    });

    expect(next.frameIndex).toBe(0);
    expect(next.lastFrameAt).toBe(250);
  });
});
