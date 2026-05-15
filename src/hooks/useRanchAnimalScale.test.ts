import { describe, expect, it } from "vitest";
import { useRanchAnimalScale } from "./useRanchAnimalScale";

describe("useRanchAnimalScale", () => {
  it("returns 1 for focuses without timers", () => {
    expect(useRanchAnimalScale(null, null)).toBe(1);
  });

  it("grows linearly from 1 to 3 over the timer duration", () => {
    expect(useRanchAnimalScale(1_000, 120, 1_060_000)).toBe(2);
  });

  it("clamps to 3 at or after the timer duration", () => {
    expect(useRanchAnimalScale(1_000, 120, 1_120_000)).toBe(3);
    expect(useRanchAnimalScale(1_000, 120, 2_000_000)).toBe(3);
  });
});
