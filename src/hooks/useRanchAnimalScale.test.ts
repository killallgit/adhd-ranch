import { describe, expect, it } from "vitest";
import { ranchAnimalScale } from "./useRanchAnimalScale";

describe("ranchAnimalScale", () => {
  it("returns 1 for focuses without timers", () => {
    expect(ranchAnimalScale(null, null)).toBe(1);
  });

  it("grows linearly from 1 to 3 over the timer duration", () => {
    expect(ranchAnimalScale(1_000, 120, 1_060_000)).toBe(2);
  });

  it("clamps to 3 at or after the timer duration", () => {
    expect(ranchAnimalScale(1_000, 120, 1_120_000)).toBe(3);
    expect(ranchAnimalScale(1_000, 120, 2_000_000)).toBe(3);
  });
});
