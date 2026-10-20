import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getDebugOverlay, getDevtoolsOpen } from "../api/debug";
import { getMonitors } from "../api/monitors";
import { getSettings, listNotificationSources, updateSettings } from "../api/settings";
import type { Settings } from "../types/settings";
import { useSettingsWindow } from "./useSettingsWindow";

vi.mock("../api/settings", () => ({
  getSettings: vi.fn(),
  listNotificationSources: vi.fn(),
  updateSettings: vi.fn(),
}));
vi.mock("../api/monitors", () => ({ getMonitors: vi.fn() }));
vi.mock("../api/debug", () => ({
  getDebugOverlay: vi.fn(),
  getDevtoolsOpen: vi.fn(),
  setDebugOverlay: vi.fn(),
  toggleDevtools: vi.fn(),
}));

const SETTINGS: Settings = {
  caps: { max_focuses: 5, max_tasks_per_focus: 7 },
  notifications: { sources: { timer_expired: false } },
  widget: { always_on_top: false, confirm_delete: true },
  displays: { enabled_indices: [0] },
  agents: { enabled: false },
  pens: { max_size: 320 },
  timers: { saved_minutes: [15] },
};

describe("useSettingsWindow notification registry", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getSettings).mockResolvedValue(SETTINGS);
    vi.mocked(getMonitors).mockResolvedValue([{ idx: 0, label: "Primary" }]);
    vi.mocked(getDebugOverlay).mockResolvedValue(true);
    vi.mocked(getDevtoolsOpen).mockResolvedValue(false);
    vi.mocked(updateSettings).mockResolvedValue();
  });

  it("loads arbitrary descriptors and sends source edits through the existing settings workflow", async () => {
    const sources = [{ key: "future_source", label: "A future notification" }];
    vi.mocked(listNotificationSources).mockResolvedValue(sources);
    const { result } = renderHook(() => useSettingsWindow());
    await waitFor(() => expect(result.current.settings).toEqual(SETTINGS));

    expect(result.current.notificationSources).toEqual(sources);
    expect(result.current.monitors).toEqual([{ idx: 0, label: "Primary" }]);
    expect(result.current.debugOverlay).toBe(true);
    const next = {
      ...SETTINGS,
      notifications: { sources: { ...SETTINGS.notifications.sources, future_source: false } },
    };

    act(() => result.current.update(next));

    expect(updateSettings).toHaveBeenCalledWith(next);
    expect(result.current.settings).toEqual(next);
  });

  it("keeps the other settings available when the registry read fails", async () => {
    const error = new Error("unavailable");
    vi.mocked(listNotificationSources).mockRejectedValue(error);
    const report = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      const { result } = renderHook(() => useSettingsWindow());

      await waitFor(() => expect(result.current.settings).toEqual(SETTINGS));

      expect(result.current.notificationSources).toEqual([]);
    } finally {
      report.mockRestore();
    }
  });
});
