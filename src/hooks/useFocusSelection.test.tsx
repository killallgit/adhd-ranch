import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Focus } from "../types/focus";
import { useFocusSelection } from "./useFocusSelection";

const detailEvent = vi.hoisted(() => ({ callback: null as null | ((id: string) => void) }));

vi.mock("../api/pig", () => ({
  subscribeOpenFocusDetail: vi.fn().mockImplementation(async (callback: (id: string) => void) => {
    detailEvent.callback = callback;
    return () => {};
  }),
}));

function focus(id: string, title = id): Focus {
  return { id, title, description: "", created_at: "", tasks: [] };
}

beforeEach(() => {
  detailEvent.callback = null;
});

describe("useFocusSelection", () => {
  it("selects current Focuses from click and tray requests", async () => {
    const { result } = renderHook(() => useFocusSelection([focus("a"), focus("b")]));
    act(() => result.current.request("a"));
    expect(result.current.selected?.id).toBe("a");

    await waitFor(() => expect(detailEvent.callback).not.toBeNull());
    act(() => detailEvent.callback?.("b"));
    expect(result.current.selected?.id).toBe("b");
  });

  it("resolves Agent render ids to no Focus", async () => {
    const { result } = renderHook(() => useFocusSelection([focus("a")]));
    act(() => result.current.request("agent:session-1"));
    await waitFor(() => expect(result.current.selected).toBeNull());
  });

  it("closes and follows updates to the current Focus", () => {
    const { result, rerender } = renderHook(({ focuses }) => useFocusSelection(focuses), {
      initialProps: { focuses: [focus("a", "Old")] },
    });
    act(() => result.current.request("a"));
    rerender({ focuses: [focus("a", "New")] });
    expect(result.current.selected?.title).toBe("New");
    act(() => result.current.close());
    expect(result.current.selected).toBeNull();
  });

  it("forgets a removed request so recreation of the same id stays closed", async () => {
    const { result, rerender } = renderHook(({ focuses }) => useFocusSelection(focuses), {
      initialProps: { focuses: [focus("a")] },
    });
    act(() => result.current.request("a"));
    rerender({ focuses: [] });
    await waitFor(() => expect(result.current.selected).toBeNull());
    rerender({ focuses: [focus("a")] });
    expect(result.current.selected).toBeNull();
  });
});
