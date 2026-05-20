import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { FocusWriter, WriteOutcome } from "../api/focusWriter";
import { useFocusController } from "./useFocusController";

const okOutcome: WriteOutcome = { ok: true };

function createWriter(overrides: Partial<FocusWriter> = {}): FocusWriter {
  const ok = vi.fn().mockResolvedValue(okOutcome);
  return {
    createFocus: ok,
    duplicateFocus: ok,
    deleteFocus: ok,
    renameFocus: ok,
    appendTask: ok,
    deleteTask: ok,
    updateTask: ok,
    toggleTask: ok,
    startTimer: ok,
    clearTimer: ok,
    startTaskTimer: ok,
    clearTaskTimer: ok,
    ...overrides,
  };
}

describe("useFocusController", () => {
  it("reports rejected writer calls as ipc failures", async () => {
    const onWriteFailure = vi.fn();
    const writer = createWriter({
      deleteFocus: vi.fn().mockRejectedValue(new Error("bridge down")),
    });

    const { result } = renderHook(() => useFocusController(writer, onWriteFailure));

    await expect(result.current.deleteFocus("focus-1")).resolves.toEqual({
      ok: false,
      kind: "ipc",
      message: "bridge down",
    });
    expect(onWriteFailure).toHaveBeenCalledWith("delete_focus", {
      ok: false,
      kind: "ipc",
      message: "bridge down",
    });
  });
});
