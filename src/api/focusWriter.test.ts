import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTauriFocusWriter } from "./focusWriter";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("tauriFocusWriter", () => {
  it("appendTask returns ok:true when invoke resolves", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    const writer = createTauriFocusWriter();

    const outcome = await writer.appendTask("focus-1", "do thing");

    expect(outcome).toEqual({ ok: true });
    expect(mockInvoke).toHaveBeenCalledWith("append_task", {
      focusId: "focus-1",
      text: "do thing",
    });
  });

  it("appendTask returns ipc failure when invoke rejects with non-CommandError", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("transport blew up"));
    const writer = createTauriFocusWriter();

    const outcome = await writer.appendTask("focus-1", "x");

    expect(outcome).toEqual({
      ok: false,
      kind: "ipc",
      message: "transport blew up",
    });
  });

  it("appendTask returns domain failure when invoke rejects with bad_request", async () => {
    mockInvoke.mockRejectedValueOnce({ type: "bad_request", message: "empty task text" });
    const writer = createTauriFocusWriter();

    const outcome = await writer.appendTask("focus-1", "");

    expect(outcome).toEqual({ ok: false, kind: "domain", message: "empty task text" });
  });

  it("appendTask returns not_found failure when invoke rejects with not_found", async () => {
    mockInvoke.mockRejectedValueOnce({ type: "not_found", message: "focus missing: x" });
    const writer = createTauriFocusWriter();

    const outcome = await writer.appendTask("missing", "y");

    expect(outcome).toEqual({ ok: false, kind: "not_found", message: "focus missing: x" });
  });
});
