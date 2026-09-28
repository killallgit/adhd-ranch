import { describe, expect, it } from "vitest";
import type { AnimalSize } from "../types/generated/AnimalSize";
import { sampleAnimalSize } from "./animalSize";

describe("sampleAnimalSize", () => {
  it("returns fixed sizes unchanged", () => {
    const size: AnimalSize = { kind: "fixed", px: 48 };
    expect(sampleAnimalSize(size, 123_456)).toBe(48);
  });

  it("clamps a linear curve before, during, and after its duration", () => {
    const size: AnimalSize = {
      kind: "linear",
      fromPx: 48,
      toPx: 96,
      startedAtMs: 1_000,
      durationMs: 2_000,
    };

    expect(sampleAnimalSize(size, 0)).toBe(48);
    expect(sampleAnimalSize(size, 2_000)).toBe(72);
    expect(sampleAnimalSize(size, 4_000)).toBe(96);
  });

  it("always returns a finite positive pixel value", () => {
    const size: AnimalSize = {
      kind: "linear",
      fromPx: 0.0001,
      toPx: 96,
      startedAtMs: 1_000,
      durationMs: 2_000,
    };
    const sampled = sampleAnimalSize(size, 2_000);

    expect(Number.isFinite(sampled)).toBe(true);
    expect(sampled).toBeGreaterThan(0);
  });

  it("accepts no zero-duration curve because Rust projects it as fixed", () => {
    const zeroDurationProjection: AnimalSize = { kind: "fixed", px: 96 };
    expect(sampleAnimalSize(zeroDurationProjection, 1_000)).toBe(96);
  });
});
