import { describe, expect, it } from "vitest";
import type { HookFiring } from "../../types/generated/HookFiring";
import { clockTime, newestFirst, shortId } from "./firings";

function firing(at: string): HookFiring {
  return { at, action: "Working", session_id: "abc", pen: "app", changed: true };
}

describe("clockTime", () => {
  it("drops the date from a timestamp", () => {
    expect(clockTime("2026-09-19T22:58:47Z")).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });

  it("shows a timestamp it cannot read rather than hiding it", () => {
    expect(clockTime("not a time")).toBe("not a time");
  });
});

describe("shortId", () => {
  it("keeps the head of a long id", () => {
    expect(shortId("4af8005a-7a52-4d0e-9f11-000000000000")).toBe("4af8005a…");
  });

  it("leaves a short id whole", () => {
    expect(shortId("abc")).toBe("abc");
  });

  it("says so when there was no id to read", () => {
    expect(shortId(null)).toBe("unreadable");
  });
});

describe("newestFirst", () => {
  it("reverses the journal's order", () => {
    const ordered = newestFirst([firing("first"), firing("second")]);

    expect(ordered.map((f) => f.at)).toEqual(["second", "first"]);
  });

  it("leaves the journal it was given untouched", () => {
    const original = [firing("first"), firing("second")];

    newestFirst(original);

    expect(original[0].at).toBe("first");
  });
});
