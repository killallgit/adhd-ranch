import { describe, expect, it } from "vitest";
import type { Focus } from "../types/focus";
import { createFixtureFocusReader } from "./fixtureFocusReader";

describe("fixtureFocusReader", () => {
  it("returns the supplied focuses", async () => {
    const focuses: Focus[] = [
      { id: "a", title: "A", description: "", tasks: [] },
      { id: "b", title: "B", description: "", tasks: [{ id: "t1", text: "do" }] },
    ];
    const reader = createFixtureFocusReader(focuses);
    expect(await reader.list()).toEqual(focuses);
  });
});
