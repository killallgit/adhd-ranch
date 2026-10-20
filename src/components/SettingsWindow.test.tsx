import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createRef } from "react";
import { describe, expect, it, vi } from "vitest";
import type { Settings } from "../types/settings";
import { SettingsWindow } from "./SettingsWindow";

const SETTINGS: Settings = {
  caps: { max_focuses: 5, max_tasks_per_focus: 7 },
  notifications: { sources: {} },
  widget: { always_on_top: false, confirm_delete: true },
  displays: { enabled_indices: [0] },
  agents: { enabled: false },
  pens: { max_size: 320 },
  timers: { saved_minutes: [15] },
};

const SOURCES = [
  { key: "timer_expired", label: "Timer expired" },
  { key: "task_timer_expired", label: "Task timer expired" },
  { key: "focuses_over_cap", label: "Too many focuses" },
  { key: "tasks_over_cap", label: "Too many tasks in a focus" },
];

function renderSettings(
  onUpdate: (next: Settings) => void,
  settings = SETTINGS,
  notificationSources = SOURCES,
) {
  render(
    <SettingsWindow
      settings={settings}
      notificationSources={notificationSources}
      monitors={[]}
      debugOverlay={false}
      devtoolsOpen={false}
      onUpdate={onUpdate}
      onSetDebugOverlay={() => {}}
      onToggleDevtools={() => {}}
      containerRef={createRef<HTMLDivElement>()}
    />,
  );
}

describe("SettingsWindow number rows", () => {
  it("keeps a half-typed pen size on screen instead of discarding it", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max pen size") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "4");

    expect(input.value).toBe("4");
    expect(onUpdate).not.toHaveBeenCalled();
  });

  it("commits a pen size typed one digit at a time", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max pen size") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "480");

    expect(onUpdate).toHaveBeenLastCalledWith(expect.objectContaining({ pens: { max_size: 480 } }));
  });

  it("corrects a pen size below the floor when the field is left", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max pen size") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "4");
    await userEvent.tab();

    expect(onUpdate).toHaveBeenLastCalledWith(expect.objectContaining({ pens: { max_size: 160 } }));
  });

  it("leaves a row whose floor is a single digit working as before", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max focuses") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "3");

    expect(onUpdate).toHaveBeenLastCalledWith(
      expect.objectContaining({ caps: { max_focuses: 3, max_tasks_per_focus: 7 } }),
    );
  });

  it("keeps saved timer minutes when another setting changes", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);

    await userEvent.click(screen.getByLabelText("Confirm Before Delete"));

    expect(onUpdate).toHaveBeenCalledWith({
      ...SETTINGS,
      widget: { ...SETTINGS.widget, confirm_delete: false },
    });
  });
});

describe("SettingsWindow notification registry", () => {
  it("renders registered labels with explicit values and enabled defaults", () => {
    renderSettings(vi.fn(), {
      ...SETTINGS,
      notifications: { sources: { timer_expired: false, focuses_over_cap: true } },
    });

    expect(screen.getByLabelText("Timer expired")).not.toBeChecked();
    expect(screen.getByLabelText("Task timer expired")).toBeChecked();
    expect(screen.getByLabelText("Too many focuses")).toBeChecked();
    expect(screen.getByLabelText("Too many tasks in a focus")).toBeChecked();
  });

  it.each([true, false])(
    "toggles an injected future source from %s and preserves settings",
    async (enabled) => {
      const onUpdate = vi.fn();
      const settings = {
        ...SETTINGS,
        notifications: { sources: { timer_expired: false, future_source: enabled } },
      };
      renderSettings(onUpdate, settings, [
        ...SOURCES,
        { key: "future_source", label: "A future notification" },
      ]);

      await userEvent.click(screen.getByLabelText("A future notification"));

      expect(onUpdate).toHaveBeenCalledWith({
        ...settings,
        notifications: { sources: { timer_expired: false, future_source: !enabled } },
      });
    },
  );

  it("does not invent notification rows before the registry loads", () => {
    renderSettings(vi.fn(), SETTINGS, []);

    expect(screen.queryByLabelText("Timer expired")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Max focuses")).toHaveValue(5);
    expect(screen.getByLabelText("Always on Top")).not.toBeChecked();
    expect(screen.getByLabelText("Max pen size")).toHaveValue(320);
    expect(screen.getByLabelText("Debug Overlay")).not.toBeChecked();
  });
});
