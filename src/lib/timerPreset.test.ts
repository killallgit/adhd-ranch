import { describe, expect, it } from "vitest";
import {
  forgetTimerMinute,
  isCustomValid,
  minutesToPreset,
  rememberTimerMinute,
} from "./timerPreset";

describe("minutesToPreset", () => {
  it("returns null when no timer is selected", () => {
    expect(minutesToPreset(null)).toBeNull();
  });

  it("wraps a minute count", () => {
    expect(minutesToPreset(15)).toEqual({ Custom: 15 });
  });
});

describe("rememberTimerMinute", () => {
  it("appends a new duration", () => {
    expect(rememberTimerMinute([15], 45)).toEqual([15, 45]);
  });

  it("leaves built-ins and duplicates out of the list", () => {
    const saved = [15];
    expect(rememberTimerMinute(saved, 10)).toBe(saved);
    expect(rememberTimerMinute(saved, 15)).toBe(saved);
    expect(rememberTimerMinute(saved, 0)).toBe(saved);
  });
});

describe("forgetTimerMinute", () => {
  it("removes one saved duration", () => {
    expect(forgetTimerMinute([15, 45], 15)).toEqual([45]);
  });

  it("returns the same list when the duration is absent", () => {
    const saved = [15];
    expect(forgetTimerMinute(saved, 45)).toBe(saved);
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
