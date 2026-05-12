import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { PresetSelection } from "../lib/timerPreset";
import { TimerPresetPicker } from "./TimerPresetPicker";

describe("TimerPresetPicker", () => {
  it("renders preset options including No timer when allowNone", () => {
    render(
      <TimerPresetPicker
        selection="none"
        customMinutes={10}
        onSelectionChange={vi.fn()}
        onCustomMinutesChange={vi.fn()}
      />,
    );
    expect(screen.getByRole("option", { name: "No timer" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Custom" })).toBeInTheDocument();
  });

  it("hides No timer when allowNone is false", () => {
    render(
      <TimerPresetPicker
        selection="Two"
        customMinutes={10}
        allowNone={false}
        onSelectionChange={vi.fn()}
        onCustomMinutesChange={vi.fn()}
      />,
    );
    expect(screen.queryByRole("option", { name: "No timer" })).not.toBeInTheDocument();
  });

  it("reveals custom input when custom selected", async () => {
    const onSelectionChange = vi.fn();
    const { rerender } = render(
      <TimerPresetPicker
        selection="none"
        customMinutes={10}
        onSelectionChange={onSelectionChange}
        onCustomMinutesChange={vi.fn()}
      />,
    );
    expect(screen.queryByTestId("custom-timer-input")).not.toBeInTheDocument();

    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "custom");
    expect(onSelectionChange).toHaveBeenCalledWith("custom" satisfies PresetSelection);

    rerender(
      <TimerPresetPicker
        selection="custom"
        customMinutes={10}
        onSelectionChange={onSelectionChange}
        onCustomMinutesChange={vi.fn()}
      />,
    );
    expect(screen.getByTestId("custom-timer-input")).toBeInTheDocument();
  });
});
