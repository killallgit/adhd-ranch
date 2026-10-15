import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TimerPresetPicker } from "./TimerPresetPicker";

function renderPicker(overrides: Partial<React.ComponentProps<typeof TimerPresetPicker>> = {}) {
  const props = {
    savedMinutes: [] as readonly number[],
    selectedMinutes: null as number | null,
    onSelect: vi.fn(),
    onAdd: vi.fn(),
    onRemove: vi.fn(),
    onClearSaved: vi.fn(),
    ...overrides,
  };
  render(<TimerPresetPicker {...props} />);
  return props;
}

describe("TimerPresetPicker", () => {
  it("shows None, the built-in durations, and add", () => {
    renderPicker();
    expect(screen.getByRole("button", { name: "None" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "10 minutes" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "30 minutes" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "60 minutes" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "add timer" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Clear saved" })).not.toBeInTheDocument();
  });

  it("hides None when allowNone is false", () => {
    renderPicker({ allowNone: false, selectedMinutes: 30 });
    expect(screen.queryByRole("button", { name: "None" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "30 minutes" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });

  it("shows saved durations after the built-ins and can remove or clear them", async () => {
    const props = renderPicker({ savedMinutes: [15, 45], selectedMinutes: 15 });
    const chips = screen.getAllByRole("button", { name: /minutes$/ });
    expect(chips.map((chip) => chip.textContent)).toEqual(["10m", "30m", "60m", "15m", "45m"]);

    await userEvent.click(screen.getByRole("button", { name: "remove 15 minute timer" }));
    expect(props.onRemove).toHaveBeenCalledWith(15);

    await userEvent.click(screen.getByRole("button", { name: "Clear saved" }));
    expect(props.onClearSaved).toHaveBeenCalled();
  });

  it("saves a new duration and selects it", async () => {
    const props = renderPicker({ selectedMinutes: 30, allowNone: false });
    await userEvent.click(screen.getByRole("button", { name: "add timer" }));
    await userEvent.type(screen.getByLabelText("custom timer minutes"), "15");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));

    expect(props.onSelect).toHaveBeenCalledWith(15);
    expect(props.onAdd).toHaveBeenCalledWith(15);
  });

  it("selects a built-in or saved duration without storing it again", async () => {
    const props = renderPicker({ savedMinutes: [15], allowNone: false });
    await userEvent.click(screen.getByRole("button", { name: "add timer" }));
    const input = screen.getByLabelText("custom timer minutes");
    await userEvent.type(input, "10");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(props.onSelect).toHaveBeenCalledWith(10);
    expect(props.onAdd).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: "add timer" }));
    await userEvent.type(screen.getByLabelText("custom timer minutes"), "15");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(props.onSelect).toHaveBeenCalledWith(15);
    expect(props.onAdd).not.toHaveBeenCalled();
  });

  it("rejects a duration under 1 minute", async () => {
    const props = renderPicker();
    await userEvent.click(screen.getByRole("button", { name: "add timer" }));
    await userEvent.type(screen.getByLabelText("custom timer minutes"), "0");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));

    expect(props.onAdd).not.toHaveBeenCalled();
    expect(props.onSelect).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent(/at least 1 minute/i);
  });

  it("drops an unconfirmed draft when asked", async () => {
    const props = {
      savedMinutes: [] as readonly number[],
      selectedMinutes: null as number | null,
      onSelect: vi.fn(),
      onAdd: vi.fn(),
      onRemove: vi.fn(),
      onClearSaved: vi.fn(),
    };
    const { rerender } = render(<TimerPresetPicker {...props} draftNonce={0} />);
    await userEvent.click(screen.getByRole("button", { name: "add timer" }));
    expect(screen.getByLabelText("custom timer minutes")).toBeInTheDocument();

    rerender(<TimerPresetPicker {...props} draftNonce={1} />);
    expect(screen.queryByLabelText("custom timer minutes")).not.toBeInTheDocument();
    expect(props.onAdd).not.toHaveBeenCalled();
  });
});
