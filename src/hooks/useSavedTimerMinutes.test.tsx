import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getSettings, updateSettings } from "../api/settings";
import type { Settings } from "../types/settings";
import { useSavedTimerMinutes } from "./useSavedTimerMinutes";

vi.mock("../api/settings", () => ({
  getSettings: vi.fn(),
  updateSettings: vi.fn(),
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
  beforeEach(() => {
    vi.mocked(getSettings).mockReset();
    vi.mocked(updateSettings).mockReset();
    vi.mocked(getSettings).mockResolvedValue(SETTINGS);
    vi.mocked(updateSettings).mockResolvedValue(undefined);
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
      expect(updateSettings).toHaveBeenCalledWith({
        ...SETTINGS,
        timers: { saved_minutes: [15, 45] },
      });
    });
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

    expect(updateSettings).not.toHaveBeenCalled();
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
      expect(updateSettings).toHaveBeenCalledWith({
        ...SETTINGS,
        timers: { saved_minutes: [] },
      });
    });
    expect(result.current.minutes).toEqual([]);
  });
});
