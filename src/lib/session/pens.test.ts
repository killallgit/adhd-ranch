import { describe, expect, it } from "vitest";
import type { Pen } from "../../types/generated/Pen";
import { layoutPens } from "./pens";

const AREA = { x: 0, y: 0, w: 1200, h: 800 };
// Larger than the area, so a test that is not about the cap never meets it.
const UNCAPPED = 10_000;

function pens(...ids: readonly string[]): readonly Pen[] {
  return ids.map((id) => ({ id, name: id.split("/").pop() ?? id }));
}

describe("layoutPens", () => {
  it("lays no pens out for an empty ranch", () => {
    expect(layoutPens([], AREA, UNCAPPED)).toEqual([]);
  });

  it("gives a single pen the whole area minus its gap", () => {
    const [only] = layoutPens(pens("/code/one"), AREA, UNCAPPED);

    expect(only.rect).toEqual({ x: 8, y: 8, w: 1184, h: 784 });
  });

  it("splits the area into one cell per pen", () => {
    const laid = layoutPens(pens("/code/a", "/code/b", "/code/c", "/code/d"), AREA, UNCAPPED);

    expect(laid).toHaveLength(4);
    expect(new Set(laid.map((p) => `${p.rect.x},${p.rect.y}`)).size).toBe(4);
  });

  it("keeps every pen inside the area", () => {
    const laid = layoutPens(pens("/a", "/b", "/c", "/d", "/e"), AREA, UNCAPPED);

    for (const { rect } of laid) {
      expect(rect.x).toBeGreaterThanOrEqual(AREA.x);
      expect(rect.y).toBeGreaterThanOrEqual(AREA.y);
      expect(rect.x + rect.w).toBeLessThanOrEqual(AREA.x + AREA.w);
      expect(rect.y + rect.h).toBeLessThanOrEqual(AREA.y + AREA.h);
    }
  });

  it("offsets the layout by the area origin", () => {
    const [only] = layoutPens(pens("/code/one"), { x: 100, y: 50, w: 400, h: 300 }, UNCAPPED);

    expect(only.rect).toEqual({ x: 108, y: 58, w: 384, h: 284 });
  });

  it("gives a pen the same cell however the roster is ordered", () => {
    const forward = layoutPens(pens("/code/a", "/code/b"), AREA, UNCAPPED);
    const reversed = layoutPens(pens("/code/b", "/code/a"), AREA, UNCAPPED);

    expect(reversed).toEqual(forward);
  });
});

describe("pen size cap", () => {
  it("stops a lone pen from becoming the whole ranch", () => {
    const [only] = layoutPens(pens("/code/one"), AREA, 320);

    expect(only.rect).toMatchObject({ w: 320, h: 320 });
  });

  it("centres a capped pen in the ranch rather than pinning it to a corner", () => {
    const [only] = layoutPens(pens("/code/one"), AREA, 320);

    expect(only.rect.x + only.rect.w / 2).toBe(AREA.w / 2);
    expect(only.rect.y + only.rect.h / 2).toBe(AREA.h / 2);
  });

  it("leaves a pen smaller than the cap alone", () => {
    const laid = layoutPens(pens("/a", "/b", "/c", "/d"), AREA, 900);

    expect(laid[0].rect).toMatchObject({ w: 584, h: 384 });
  });

  it("keeps capped pens side by side without overlapping", () => {
    const [first, second] = layoutPens(pens("/code/a", "/code/b"), AREA, 200);

    expect(first.rect.x + first.rect.w).toBeLessThanOrEqual(second.rect.x);
  });
});

describe("pen colour", () => {
  it("gives one pen the same hue every time", () => {
    const first = layoutPens(pens("/code/one"), AREA, UNCAPPED);
    const again = layoutPens(pens("/code/one"), AREA, UNCAPPED);

    expect(again[0].hue).toBe(first[0].hue);
  });

  it("stays within the colour wheel", () => {
    const laid = layoutPens(pens("/a", "/b", "/c", "/code/long/path"), AREA, UNCAPPED);

    for (const { hue } of laid) {
      expect(hue).toBeGreaterThanOrEqual(0);
      expect(hue).toBeLessThan(360);
    }
  });
});
