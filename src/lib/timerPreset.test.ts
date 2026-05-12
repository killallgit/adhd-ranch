import { describe, expect, it } from "vitest";
import { isCustomValid, resolvePreset } from "./timerPreset";

describe("resolvePreset", () => {
  it("returns null for none", () => {
    expect(resolvePreset("none", 10)).toBeNull();
  });

  it("returns named presets verbatim", () => {
    expect(resolvePreset("Eight", 10)).toBe("Eight");
    expect(resolvePreset("ThirtyTwo", 10)).toBe("ThirtyTwo");
  });

  it("wraps custom minutes", () => {
    expect(resolvePreset("custom", 15)).toEqual({ Custom: 15 });
  });
});

describe("isCustomValid", () => {
  it("rejects sub-minute values", () => {
    expect(isCustomValid(0)).toBe(false);
    expect(isCustomValid(-1)).toBe(false);
  });

  it("rejects non-integer or non-finite", () => {
    expect(isCustomValid(1.5)).toBe(false);
    expect(isCustomValid(Number.NaN)).toBe(false);
    expect(isCustomValid(Number.POSITIVE_INFINITY)).toBe(false);
  });

  it("accepts integers >= 1", () => {
    expect(isCustomValid(1)).toBe(true);
    expect(isCustomValid(120)).toBe(true);
  });
});
