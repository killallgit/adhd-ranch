import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { editSavedTimerMinutes, getSettings } from "../api/settings";
import { forgetTimerMinute, rememberTimerMinute } from "../lib/timerPreset";
import type { Settings } from "../types/settings";
import { useSavedTimerMinutes } from "./useSavedTimerMinutes";

vi.mock("../api/settings", () => ({
  getSettings: vi.fn(),
  editSavedTimerMinutes: vi.fn(),
}));

const SETTINGS: Settings = {
  caps: { max_focuses: 5, max_tasks_per_focus: 7 },
  notifications: { sources: {} },
  widget: { always_on_top: false, confirm_delete: true },
  displays: { enabled_indices: [0] },
  agents: { enabled: false },
  pens: { max_size: 320 },
  timers: { saved_minutes: [15] },
};

describe("useSavedTimerMinutes", () => {
  let stored: Settings;

  beforeEach(() => {
    stored = SETTINGS;
    vi.mocked(getSettings).mockReset();
    vi.mocked(editSavedTimerMinutes).mockReset();
    vi.mocked(getSettings).mockImplementation(async () => stored);
    vi.mocked(editSavedTimerMinutes).mockImplementation(async (edit) => {
      const current = stored.timers.saved_minutes;
      const next =
        edit === "Clear"
          ? []
          : "Add" in edit
            ? rememberTimerMinute(current, edit.Add)
            : forgetTimerMinute(current, edit.Remove);
      stored = { ...stored, timers: { saved_minutes: [...next] } };
      return stored.timers;
    });
  });

  it("loads saved minutes", async () => {
    const { result } = renderHook(() => useSavedTimerMinutes());

    await waitFor(() => {
      expect(result.current.minutes).toEqual([15]);
    });
  });

  it("adds a duration without dropping the rest of settings", async () => {
    const { result } = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => {
      expect(result.current.minutes).toEqual([15]);
    });

    act(() => {
      result.current.add(45);
    });

    await waitFor(() => {
      expect(stored.timers.saved_minutes).toEqual([15, 45]);
    });
    expect(editSavedTimerMinutes).toHaveBeenCalledWith({ Add: 45 });
    expect(stored).toEqual({ ...SETTINGS, timers: { saved_minutes: [15, 45] } });
    expect(result.current.minutes).toEqual([15, 45]);
  });

  it("does not store a built-in duration", async () => {
    const { result } = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => {
      expect(result.current.minutes).toEqual([15]);
    });

    act(() => {
      result.current.add(10);
    });

    expect(editSavedTimerMinutes).not.toHaveBeenCalled();
    expect(result.current.minutes).toEqual([15]);
  });

  it("clears saved minutes and leaves built-ins to the picker", async () => {
    const { result } = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => {
      expect(result.current.minutes).toEqual([15]);
    });

    act(() => {
      result.current.clear();
    });

    await waitFor(() => {
      expect(stored.timers.saved_minutes).toEqual([]);
    });
    expect(editSavedTimerMinutes).toHaveBeenCalledWith("Clear");
    expect(result.current.minutes).toEqual([]);
  });

  it("keeps rapid additions made before another render", async () => {
    const { result } = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => expect(result.current.minutes).toEqual([15]));

    act(() => {
      result.current.add(25);
      result.current.add(45);
    });

    await waitFor(() => {
      expect(stored.timers.saved_minutes).toEqual([15, 25, 45]);
    });
    expect(result.current.minutes).toEqual([15, 25, 45]);
  });

  it("preserves a duration saved by another window with a stale local list", async () => {
    const first = renderHook(() => useSavedTimerMinutes());
    const second = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => {
      expect(first.result.current.minutes).toEqual([15]);
      expect(second.result.current.minutes).toEqual([15]);
    });

    act(() => first.result.current.add(25));
    await waitFor(() => expect(stored.timers.saved_minutes).toEqual([15, 25]));
    act(() => second.result.current.add(45));

    await waitFor(() => expect(stored.timers.saved_minutes).toEqual([15, 25, 45]));
    expect(second.result.current.minutes).toEqual([15, 25, 45]);
  });

  it("sends concurrent edits from separate windows without stale snapshots", async () => {
    const first = renderHook(() => useSavedTimerMinutes());
    const second = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => {
      expect(first.result.current.minutes).toEqual([15]);
      expect(second.result.current.minutes).toEqual([15]);
    });

    act(() => {
      first.result.current.add(25);
      second.result.current.add(45);
    });

    await waitFor(() => expect(stored.timers.saved_minutes).toEqual([15, 25, 45]));
    expect(editSavedTimerMinutes).toHaveBeenCalledWith({ Add: 25 });
    expect(editSavedTimerMinutes).toHaveBeenCalledWith({ Add: 45 });
  });

  it("removes a duration without losing a newer save from another window", async () => {
    const { result } = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => expect(result.current.minutes).toEqual([15]));
    stored = { ...SETTINGS, timers: { saved_minutes: [15, 25] } };

    act(() => result.current.remove(15));

    await waitFor(() => expect(result.current.minutes).toEqual([25]));
    expect(editSavedTimerMinutes).toHaveBeenCalledWith({ Remove: 15 });
    expect(stored.timers.saved_minutes).toEqual([25]);
  });

  it("does not replace a newer optimistic edit with an older save response", async () => {
    const { result } = renderHook(() => useSavedTimerMinutes());
    await waitFor(() => expect(result.current.minutes).toEqual([15]));
    let finishFirst: (value: { saved_minutes: number[] }) => void = () => {};
    let finishSecond: (value: { saved_minutes: number[] }) => void = () => {};
    vi.mocked(editSavedTimerMinutes)
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            finishFirst = resolve;
          }),
      )
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            finishSecond = resolve;
          }),
      );

    act(() => {
      result.current.add(25);
      result.current.add(45);
    });
    await waitFor(() => expect(editSavedTimerMinutes).toHaveBeenCalledTimes(1));
    await act(async () => finishFirst({ saved_minutes: [15, 25] }));

    expect(result.current.minutes).toEqual([15, 25, 45]);
    await act(async () => finishSecond({ saved_minutes: [15, 25, 45] }));
    expect(result.current.minutes).toEqual([15, 25, 45]);
  });
});
