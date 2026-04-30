import { describe, expect, it } from "vitest";
import type { Focus } from "../types/focus";
import { computeCapState } from "./capState";

const caps = { max_focuses: 5, max_tasks_per_focus: 7 };

function makeFocus(id: string, taskCount: number): Focus {
  return {
    id,
    title: id,
    description: "",
    tasks: Array.from({ length: taskCount }, (_, i) => ({ id: `${id}:${i}`, text: `t${i}` })),
  };
}

describe("computeCapState", () => {
  it("returns no flags when under limits", () => {
    const state = computeCapState([makeFocus("a", 3)], caps);
    expect(state.focusesOver).toBe(false);
    expect(state.overTaskFocusIds).toEqual([]);
    expect(state.anyOver).toBe(false);
  });

  it("flags when focuses count exceeds max", () => {
    const focuses = Array.from({ length: 6 }, (_, i) => makeFocus(`f${i}`, 0));
    const state = computeCapState(focuses, caps);
    expect(state.focusesOver).toBe(true);
    expect(state.focusCount).toBe(6);
    expect(state.anyOver).toBe(true);
  });

  it("lists over-tasks focus ids", () => {
    const state = computeCapState(
      [makeFocus("good", 3), makeFocus("bad", 9), makeFocus("worse", 12)],
      caps,
    );
    expect(state.focusesOver).toBe(false);
    expect(state.overTaskFocusIds).toEqual(["bad", "worse"]);
    expect(state.anyOver).toBe(true);
  });

  it("at-cap is not over", () => {
    const focuses = Array.from({ length: 5 }, (_, i) => makeFocus(`f${i}`, 7));
    const state = computeCapState(focuses, caps);
    expect(state.focusesOver).toBe(false);
    expect(state.overTaskFocusIds).toEqual([]);
  });
});
