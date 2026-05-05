import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import type { FocusReader } from "../api/focuses";
import type { Focus } from "../types/focus";
import { useFocuses } from "./useFocuses";

describe("useFocuses", () => {
  it("transitions loading → ready with focuses", async () => {
    const reader = createFixtureFocusReader([
      { id: "a", title: "A", description: "", created_at: "", tasks: [] },
    ]);
    const { result } = renderHook(() => useFocuses(reader));
    expect(result.current.status).toBe("loading");
    await waitFor(() => {
      expect(result.current.status).toBe("ready");
    });
    if (result.current.status === "ready") {
      expect(result.current.focuses).toHaveLength(1);
      expect(result.current.focuses[0]?.id).toBe("a");
    }
  });

  it("re-fetches when subscribe callback fires", async () => {
    let callCount = 0;
    let trigger: (() => void) | null = null;
    const focusesA: Focus[] = [{ id: "a", title: "A", description: "", created_at: "", tasks: [] }];
    const focusesB: Focus[] = [
      { id: "a", title: "A", description: "", created_at: "", tasks: [] },
      { id: "b", title: "B", description: "", created_at: "", tasks: [] },
    ];
    const reader: FocusReader = {
      list: () => {
        callCount += 1;
        return Promise.resolve(callCount === 1 ? focusesA : focusesB);
      },
      subscribe: (onChange) => {
        trigger = onChange;
        return () => {
          trigger = null;
        };
      },
    };

    const { result } = renderHook(() => useFocuses(reader));
    await waitFor(() => {
      expect(result.current.status).toBe("ready");
    });
    if (result.current.status === "ready") {
      expect(result.current.focuses).toHaveLength(1);
    }

    await act(async () => {
      trigger?.();
    });

    await waitFor(() => {
      if (result.current.status !== "ready") throw new Error("not ready");
      expect(result.current.focuses).toHaveLength(2);
    });
  });
});
