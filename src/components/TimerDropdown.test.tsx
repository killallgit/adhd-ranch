import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TimerDropdown } from "./TimerDropdown";

function dropdownProps(overrides: Partial<React.ComponentProps<typeof TimerDropdown>> = {}) {
  return {
    ariaLabel: "edit timer",
    savedMinutes: [] as readonly number[],
    onStart: vi.fn(),
    onAddSaved: vi.fn(),
    onRemoveSaved: vi.fn(),
    onClearSaved: vi.fn(),
    ...overrides,
  };
}

function renderDropdown(overrides: Partial<React.ComponentProps<typeof TimerDropdown>> = {}) {
  const props = dropdownProps(overrides);
  render(<TimerDropdown {...props} />);
  return props;
}

describe("TimerDropdown", () => {
  it("starts the default 30 minute chip when Enter is pressed", async () => {
    const props = renderDropdown();

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.keyboard("{Enter}");

    expect(props.onStart).toHaveBeenCalledWith({ Custom: 30 });
  });

  it("starts the selected chip when focus leaves the dropdown", async () => {
    const props = dropdownProps();
    render(
      <>
        <TimerDropdown {...props} />
        <button type="button">outside</button>
      </>,
    );

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.click(screen.getByRole("button", { name: "60 minutes" }));
    await userEvent.click(screen.getByRole("button", { name: "outside" }));

    expect(props.onStart).toHaveBeenCalledTimes(1);
    expect(props.onStart).toHaveBeenCalledWith({ Custom: 60 });
  });

  it("does not start when an outside click dismisses an unconfirmed duration", async () => {
    const props = dropdownProps();
    render(
      <>
        <TimerDropdown {...props} />
        <button type="button">outside</button>
      </>,
    );

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.click(screen.getByRole("button", { name: "add timer" }));
    await userEvent.type(screen.getByLabelText("custom timer minutes"), "25");
    await userEvent.click(screen.getByRole("button", { name: "outside" }));

    expect(props.onStart).not.toHaveBeenCalled();
    expect(props.onAddSaved).not.toHaveBeenCalled();
    expect(screen.queryByLabelText("custom timer minutes")).not.toBeInTheDocument();
  });

  it("saves a new duration and starts it", async () => {
    const props = renderDropdown();

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.click(screen.getByRole("button", { name: "add timer" }));
    await userEvent.type(screen.getByLabelText("custom timer minutes"), "25");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));
    await userEvent.click(screen.getByRole("button", { name: "Start" }));

    expect(props.onAddSaved).toHaveBeenCalledWith(25);
    expect(props.onStart).toHaveBeenCalledTimes(1);
    expect(props.onStart).toHaveBeenCalledWith({ Custom: 25 });
  });

  it("does not auto-commit when moving focus inside the dropdown", async () => {
    const props = renderDropdown();

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.click(screen.getByRole("button", { name: "60 minutes" }));
    await userEvent.click(screen.getByRole("button", { name: "Start" }));

    expect(props.onStart).toHaveBeenCalledTimes(1);
    expect(props.onStart).toHaveBeenCalledWith({ Custom: 60 });
  });
});
