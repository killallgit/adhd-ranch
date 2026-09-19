import { describe, expect, it } from "vitest";
import type { Pen } from "../../types/generated/Pen";
import { layoutPens } from "./pens";

const AREA = { x: 0, y: 0, w: 1200, h: 800 };

function pens(...ids: readonly string[]): readonly Pen[] {
  return ids.map((id) => ({ id, name: id.split("/").pop() ?? id }));
}

describe("layoutPens", () => {
  it("lays no pens out for an empty ranch", () => {
    expect(layoutPens([], AREA)).toEqual([]);
  });

  it("gives a single pen the whole area minus its gap", () => {
    const [only] = layoutPens(pens("/code/one"), AREA);

    expect(only.rect).toEqual({ x: 8, y: 8, w: 1184, h: 784 });
  });

  it("splits the area into one cell per pen", () => {
    const laid = layoutPens(pens("/code/a", "/code/b", "/code/c", "/code/d"), AREA);

    expect(laid).toHaveLength(4);
    expect(new Set(laid.map((p) => `${p.rect.x},${p.rect.y}`)).size).toBe(4);
  });

  it("keeps every pen inside the area", () => {
    const laid = layoutPens(pens("/a", "/b", "/c", "/d", "/e"), AREA);

    for (const { rect } of laid) {
      expect(rect.x).toBeGreaterThanOrEqual(AREA.x);
      expect(rect.y).toBeGreaterThanOrEqual(AREA.y);
      expect(rect.x + rect.w).toBeLessThanOrEqual(AREA.x + AREA.w);
      expect(rect.y + rect.h).toBeLessThanOrEqual(AREA.y + AREA.h);
    }
  });

  it("offsets the layout by the area origin", () => {
    const [only] = layoutPens(pens("/code/one"), { x: 100, y: 50, w: 400, h: 300 });

    expect(only.rect).toEqual({ x: 108, y: 58, w: 384, h: 284 });
  });

  it("gives a pen the same cell however the roster is ordered", () => {
    const forward = layoutPens(pens("/code/a", "/code/b"), AREA);
    const reversed = layoutPens(pens("/code/b", "/code/a"), AREA);

    expect(reversed).toEqual(forward);
  });
});

describe("pen colour", () => {
  it("gives one pen the same hue every time", () => {
    const first = layoutPens(pens("/code/one"), AREA);
    const again = layoutPens(pens("/code/one"), AREA);

    expect(again[0].hue).toBe(first[0].hue);
  });

  it("stays within the colour wheel", () => {
    const laid = layoutPens(pens("/a", "/b", "/c", "/code/long/path"), AREA);

    for (const { hue } of laid) {
      expect(hue).toBeGreaterThanOrEqual(0);
      expect(hue).toBeLessThan(360);
    }
  });
});
