import { describe, expect, it } from "vitest";
import { createFixtureFocusWriter } from "./fixtureFocusWriter";

describe("fixtureFocusWriter", () => {
  it("returns ok:true by default", async () => {
    const writer = createFixtureFocusWriter();

    expect(await writer.appendTask("a", "x")).toEqual({ ok: true });
    expect(await writer.createFocus({ title: "t" })).toEqual({ ok: true });
  });

  it("returns the injected failure", async () => {
    const writer = createFixtureFocusWriter({
      failure: { ok: false, kind: "domain", message: "boom" },
    });

    expect(await writer.appendTask("a", "x")).toEqual({
      ok: false,
      kind: "domain",
      message: "boom",
    });
    expect(await writer.renameFocus("a", "n")).toEqual({
      ok: false,
      kind: "domain",
      message: "boom",
    });
  });
});
