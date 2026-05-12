import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  type PresetSelection,
  TimerPresetPicker,
  isCustomValid,
  resolvePreset,
} from "./TimerPresetPicker";

describe("resolvePreset", () => {
  it("returns null for none", () => {
    expect(resolvePreset("none", 10)).toBeNull();
  });

  it("returns named presets verbatim", () => {
    expect(resolvePreset("Eight", 10)).toBe("Eight");
    expect(resolvePreset("ThirtyTwo", 10)).toBe("ThirtyTwo");
  });

  it("wraps custom minutes", () => {
    expect(resolvePreset("custom", 15)).toEqual({ Custom: 15 });
  });
});

describe("isCustomValid", () => {
  it("rejects sub-minute values", () => {
    expect(isCustomValid(0)).toBe(false);
    expect(isCustomValid(-1)).toBe(false);
  });

  it("rejects non-integer or non-finite", () => {
    expect(isCustomValid(1.5)).toBe(false);
    expect(isCustomValid(Number.NaN)).toBe(false);
    expect(isCustomValid(Number.POSITIVE_INFINITY)).toBe(false);
  });

  it("accepts integers >= 1", () => {
    expect(isCustomValid(1)).toBe(true);
    expect(isCustomValid(120)).toBe(true);
  });
});

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
