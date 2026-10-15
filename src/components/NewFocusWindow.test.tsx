import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { NewFocusWindow } from "./NewFocusWindow";

function renderWindow(selectedMinutes: number | null) {
  const props = {
    title: "Focus",
    description: "",
    selectedMinutes,
    savedTimers: {
      minutes: [15, 45],
      add: vi.fn(),
      remove: vi.fn(),
      clear: vi.fn(),
      reload: vi.fn(),
    },
    submitting: false,
    error: null,
    onTitleChange: vi.fn(),
    onDescriptionChange: vi.fn(),
    onSelectMinutes: vi.fn(),
    onSubmit: vi.fn(),
    onCancel: vi.fn(),
  };
  render(<NewFocusWindow {...props} />);
  return props;
}

describe("NewFocusWindow", () => {
  it("clears the selected duration when its saved chip is removed", async () => {
    const props = renderWindow(15);

    await userEvent.click(screen.getByRole("button", { name: "remove 15 minute timer" }));

    expect(props.savedTimers.remove).toHaveBeenCalledWith(15);
    expect(props.onSelectMinutes).toHaveBeenCalledWith(null);
  });

  it("keeps the selected duration when a different chip is removed", async () => {
    const props = renderWindow(45);

    await userEvent.click(screen.getByRole("button", { name: "remove 15 minute timer" }));

    expect(props.savedTimers.remove).toHaveBeenCalledWith(15);
    expect(props.onSelectMinutes).not.toHaveBeenCalled();
  });

  it("clears the selected saved duration when saved chips are cleared", async () => {
    const props = renderWindow(15);

    await userEvent.click(screen.getByRole("button", { name: "Clear saved" }));

    expect(props.savedTimers.clear).toHaveBeenCalled();
    expect(props.onSelectMinutes).toHaveBeenCalledWith(null);
  });

  it("keeps a selected built-in duration when saved chips are cleared", async () => {
    const props = renderWindow(30);

    await userEvent.click(screen.getByRole("button", { name: "Clear saved" }));

    expect(props.savedTimers.clear).toHaveBeenCalled();
    expect(props.onSelectMinutes).not.toHaveBeenCalled();
  });
});
