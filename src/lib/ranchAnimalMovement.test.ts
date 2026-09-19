import { describe, expect, it } from "vitest";
import type { Rect } from "../types/display";
import {
  RANCH_ANIMAL_SIZE,
  type RanchAnimalState,
  advanceRanchAnimal,
  edgeMargin,
  restRanchAnimal,
} from "./ranchAnimalMovement";

const ranchRegions: readonly Rect[] = [
  { x: 1280, y: 0, w: 1920, h: 1080 },
  { x: 0, y: 0, w: 1280, h: 800 },
];

const pen: Rect = { x: 600, y: 300, w: 240, h: 200 };
const neighbouringPen: Rect = { x: 900, y: 300, w: 240, h: 200 };

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

function seededRandom(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1664525 + 1013904223) % 4294967296;
    return state / 4294967296;
  };
}

function roam(
  start: RanchAnimalState,
  regions: readonly Rect[],
  ticks: number,
): readonly RanchAnimalState[] {
  const random = seededRandom(7);
  const path: RanchAnimalState[] = [];
  let current = start;
  for (let step = 1; step <= ticks; step += 1) {
    current = advanceRanchAnimal({
      animal: current,
      regions,
      dtMs: 100,
      nowMs: step * 100,
      frozen: false,
      random,
    });
    path.push(current);
  }
  return path;
}

describe("edgeMargin", () => {
  it("shrinks to a quarter of a small pen's shorter side", () => {
    expect(edgeMargin({ x: 0, y: 0, w: 160, h: 80 })).toBe(20);
  });

  it("caps at the ranch margin for a region larger than four times it", () => {
    expect(edgeMargin({ x: 0, y: 0, w: 1920, h: 1080 })).toBe(60);
  });
});

describe("advanceRanchAnimal", () => {
  it("clamps an invalid position to the nearest movement region", () => {
    const next = advanceRanchAnimal({
      animal: animal({ x: 1000, y: 900, vx: 0, vy: 0 }),
      regions: ranchRegions,
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
      regions: [{ x: 0, y: 0, w: 200, h: 200 }],
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
      regions: [{ x: 0, y: 0, w: 200, h: 200 }],
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
      regions: [{ x: 0, y: 0, w: 400, h: 400 }],
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
      regions: ranchRegions,
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
      regions: [{ x: 0, y: 0, w: 600, h: 600 }],
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
      regions: [{ x: 0, y: 0, w: 600, h: 600 }],
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
      regions: [{ x: 0, y: 0, w: 600, h: 600 }],
      dtMs: 0,
      nowMs: 250,
      frozen: false,
      random: () => 0,
    });

    expect(next.frameIndex).toBe(0);
    expect(next.lastFrameAt).toBe(250);
  });

  it("keeps an animal inside the small pen it was given over many ticks", () => {
    const path = roam(animal({ x: 700, y: 400, vx: 240, vy: 180 }), [pen], 200);

    const xs = path.map((state) => state.x);
    const ys = path.map((state) => state.y);
    expect(Math.min(...xs)).toBeGreaterThanOrEqual(pen.x);
    expect(Math.max(...xs)).toBeLessThanOrEqual(pen.x + pen.w - RANCH_ANIMAL_SIZE);
    expect(Math.min(...ys)).toBeGreaterThanOrEqual(pen.y);
    expect(Math.max(...ys)).toBeLessThanOrEqual(pen.y + pen.h - RANCH_ANIMAL_SIZE);
  });

  it("never crosses into a neighbouring region it was not given", () => {
    const path = roam(animal({ x: 700, y: 400, vx: 240, vy: 180 }), [pen], 200);

    const trespassing = path.filter((state) => state.x + RANCH_ANIMAL_SIZE > neighbouringPen.x);
    expect(trespassing).toHaveLength(0);
  });
});

describe("restRanchAnimal", () => {
  it("stops the animal and faces it away", () => {
    const resting = restRanchAnimal(animal({ x: 650, y: 350 }), [pen]);

    expect(resting).toMatchObject({ vx: 0, vy: 0, direction: "back" });
  });

  it("leaves an animal already resting inside its pen untouched", () => {
    const resting = restRanchAnimal(animal({ x: 650, y: 350 }), [pen]);

    expect(restRanchAnimal(resting, [pen])).toBe(resting);
  });

  it("pulls a resting animal back into the pen when the pen moves out from under it", () => {
    const stranded = restRanchAnimal(animal({ x: 1000, y: 350 }), [neighbouringPen]);

    const reconfined = restRanchAnimal(stranded, [pen]);

    expect(reconfined.x).toBeLessThanOrEqual(pen.x + pen.w - RANCH_ANIMAL_SIZE);
    expect(reconfined.x).toBeGreaterThanOrEqual(pen.x);
  });

  it("parks an animal at the corner of a pen too small to hold it", () => {
    const cramped: Rect = { x: 600, y: 300, w: 20, h: 20 };

    const resting = restRanchAnimal(animal({ x: 0, y: 0 }), [cramped]);

    expect(resting).toMatchObject({ x: cramped.x, y: cramped.y });
  });
});
