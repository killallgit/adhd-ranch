import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TimerPresetEdit } from "../types/generated/TimerPresetEdit";
import { editSavedTimerMinutes, listNotificationSources } from "./settings";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("editSavedTimerMinutes", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it.each<TimerPresetEdit>([{ Add: 25 }, { Remove: 15 }, "Clear"])(
    "sends a timer edit and returns the committed presets: %j",
    async (edit) => {
      const saved = { saved_minutes: [25] };
      vi.mocked(invoke).mockResolvedValue(saved);

      const result = await editSavedTimerMinutes(edit);

      expect(invoke).toHaveBeenCalledWith("edit_saved_timer_minutes", { edit });
      expect(result).toEqual(saved);
    },
  );

  it("propagates persistence failures", async () => {
    const error = new Error("unavailable");
    vi.mocked(invoke).mockRejectedValue(error);

    await expect(editSavedTimerMinutes({ Add: 25 })).rejects.toBe(error);
  });
});

describe("listNotificationSources", () => {
  it("returns registry descriptors from IPC without filtering future keys", async () => {
    const sources = [{ key: "future_source", label: "A future notification" }];
    vi.mocked(invoke).mockResolvedValue(sources);

    expect(await listNotificationSources()).toEqual(sources);
    expect(invoke).toHaveBeenCalledWith("list_notification_sources");
  });

  it("propagates registry read failures", async () => {
    const error = new Error("unavailable");
    vi.mocked(invoke).mockRejectedValue(error);

    await expect(listNotificationSources()).rejects.toBe(error);
  });
});
