import { describe, expect, it } from "vitest";
import type { Animal } from "../types/generated/Animal";
import type { SpeciesProfile } from "../types/generated/SpeciesProfile";
import {
  type AnimalMovementState,
  advanceAnimal,
  computeTossVelocity,
  reconcileAnimalMovement,
  spawnAnimal,
} from "./animalMovement";
import { movementRegionsFor, spawnRegionFor } from "./regions";

const PROFILE: SpeciesProfile = {
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
};

const REGION = { x: 0, y: 0, w: 400, h: 300 };

function animal(overrides: Partial<Animal> = {}): Animal {
  return {
    id: "focus-1",
    label: "Focus",
    species: "pig",
    size: { kind: "fixed", px: 48 },
    regionId: null,
    motion: "walking",
    ...overrides,
  };
}

function state(overrides: Partial<AnimalMovementState> = {}): AnimalMovementState {
  return {
    id: "focus-1",
    label: "Focus",
    x: 100,
    y: 100,
    vx: 60,
    vy: 0,
    frameIndex: 0,
    direction: "right",
    lastFrameAt: 0,
    nextTurnAt: 10_000,
    ...overrides,
  };
}

describe("animalMovement", () => {
  it("spawns generated Animal ids with SpeciesProfile-supplied physics", () => {
    const spawned = spawnAnimal(animal({ id: "agent:session" }), REGION, PROFILE, 1_000, () => 0.5);

    expect(spawned.id).toBe("agent:session");
    expect(Math.hypot(spawned.vx, spawned.vy)).toBeCloseTo(PROFILE.walkSpeedPxPerSecond);
    expect(spawned.nextTurnAt).toBe(1_000 + PROFILE.turnMinMs + 2_500);
  });

  it("walks, rests, and freezes by id without source-domain knowledge", () => {
    const walking = advanceAnimal({
      animal: state(),
      regions: [REGION],
      dtMs: 100,
      nowMs: 100,
      motion: "walking",
      frozen: false,
      random: () => 0.5,
      profile: PROFILE,
    });
    expect(walking.x).toBeGreaterThan(100);

    const resting = advanceAnimal({
      animal: walking,
      regions: [REGION],
      dtMs: 100,
      nowMs: 200,
      motion: "resting",
      frozen: false,
      random: () => 0.5,
      profile: PROFILE,
    });
    expect(resting).toMatchObject({ vx: 0, vy: 0, direction: "back" });

    const frozen = advanceAnimal({
      animal: state(),
      regions: [REGION],
      dtMs: 100,
      nowMs: 100,
      motion: "walking",
      frozen: true,
      random: () => 0.5,
      profile: PROFILE,
    });
    expect(frozen).toMatchObject({ x: 100, y: 100, vx: 60, vy: 0 });
  });

  it("keeps a frozen resting animal at its selected position", () => {
    const selected = state({ x: -20, y: -30 });

    const frozen = advanceAnimal({
      animal: selected,
      regions: [REGION],
      dtMs: 100,
      nowMs: 100,
      motion: "resting",
      frozen: true,
      random: () => 0.5,
      profile: PROFILE,
    });

    expect(frozen).toMatchObject({
      x: selected.x,
      y: selected.y,
      vx: selected.vx,
      vy: selected.vy,
    });
  });

  it("preserves position when label, size, or Motion presentation changes", () => {
    const existing = state({ x: 123, y: 234 });
    const reconciled = reconcileAnimalMovement(
      existing,
      animal({
        label: "Updated",
        size: { kind: "linear", fromPx: 48, toPx: 144, startedAtMs: 0, durationMs: 100 },
        motion: "resting",
      }),
      REGION,
      PROFILE,
      1_000,
      () => 0.5,
    );

    expect(reconciled).toMatchObject({ x: 123, y: 234, label: "Updated" });
  });

  it("clamps drag toss velocity using the supplied SpeciesProfile", () => {
    const velocity = computeTossVelocity(
      [
        { x: 0, y: 0, t: 990 },
        { x: 10_000, y: 0, t: 1_000 },
      ],
      80,
      1_000,
      PROFILE,
    );

    expect(velocity.vx).toBe(PROFILE.walkSpeedPxPerSecond * PROFILE.maximumTossSpeedMultiplier);
    expect(velocity.vy).toBe(0);
  });

  it("uses DisplaySpace for null and unresolved region ids", () => {
    const display = {
      span: { w: 400, h: 300 },
      spawnRegion: REGION,
      movementRegions: [REGION],
      hitTestScale: 1,
    };
    const sceneRegions = [
      { id: "repo", label: "Repo", rect: { x: 50, y: 60, w: 200, h: 150 }, hue: 12 },
    ];

    expect(movementRegionsFor(animal({ regionId: null }), sceneRegions, display)).toEqual([REGION]);
    expect(spawnRegionFor(animal({ regionId: "missing" }), sceneRegions, display)).toEqual(REGION);
    expect(movementRegionsFor(animal({ regionId: "missing" }), [], display)).toEqual([REGION]);
  });

  it("uses known scene rectangles and reconfines after a region changes", () => {
    const display = {
      span: { w: 800, h: 600 },
      spawnRegion: { x: 0, y: 0, w: 800, h: 600 },
      movementRegions: [{ x: 0, y: 0, w: 800, h: 600 }],
      hitTestScale: 1,
    };
    const newRegion = { x: 300, y: 200, w: 160, h: 160 };
    const layouts = [{ id: "repo", label: "Repo", rect: newRegion, hue: 12 }];
    const source = animal({ regionId: "repo", motion: "resting" });

    expect(spawnRegionFor(source, layouts, display)).toEqual(newRegion);
    const reconfined = advanceAnimal({
      animal: state({ x: 20, y: 20 }),
      regions: movementRegionsFor(source, layouts, display),
      dtMs: 0,
      nowMs: 0,
      motion: "resting",
      frozen: false,
      random: () => 0.5,
      profile: PROFILE,
    });
    expect(reconfined.x).toBe(newRegion.x);
    expect(reconfined.y).toBe(newRegion.y);
  });

  it("preserves the current Pig speed, turn, friction, and toss policy", () => {
    expect(PROFILE).toMatchObject({
      walkSpeedPxPerSecond: 60,
      friction: 0.97,
      minimumSpeedFraction: 0.35,
      maximumTossSpeedMultiplier: 6,
      turnMinMs: 3_000,
      turnMaxMs: 8_000,
      frameIntervalMs: 150,
    });
  });
});
