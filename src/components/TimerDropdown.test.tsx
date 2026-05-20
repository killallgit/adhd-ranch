import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TimerDropdown } from "./TimerDropdown";

describe("TimerDropdown", () => {
  it("commits the selected preset when Enter is pressed", async () => {
    const onStart = vi.fn();
    render(<TimerDropdown ariaLabel="edit timer" onStart={onStart} />);

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "ThirtyTwo");
    await userEvent.keyboard("{Enter}");

    expect(onStart).toHaveBeenCalledWith("ThirtyTwo");
  });

  it("commits custom minutes when focus leaves the dropdown", async () => {
    const onStart = vi.fn();
    render(
      <>
        <TimerDropdown ariaLabel="edit timer" onStart={onStart} />
        <button type="button">outside</button>
      </>,
    );

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "custom");
    const input = screen.getByTestId("custom-timer-input");
    await userEvent.clear(input);
    await userEvent.type(input, "25");
    await userEvent.click(screen.getByRole("button", { name: "outside" }));

    expect(onStart).toHaveBeenCalledWith({ Custom: 25 });
  });

  it("commits only once when an outside click also blurs the dropdown", async () => {
    const onStart = vi.fn();
    render(
      <>
        <TimerDropdown ariaLabel="edit timer" onStart={onStart} />
        <button type="button">outside</button>
      </>,
    );

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "ThirtyTwo");
    await userEvent.click(screen.getByRole("button", { name: "outside" }));

    expect(onStart).toHaveBeenCalledTimes(1);
    expect(onStart).toHaveBeenCalledWith("ThirtyTwo");
  });

  it("does not auto-commit when moving focus inside the dropdown", async () => {
    const onStart = vi.fn();
    render(<TimerDropdown ariaLabel="edit timer" onStart={onStart} />);

    await userEvent.click(screen.getByRole("button", { name: "edit timer" }));
    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "ThirtyTwo");
    await userEvent.click(screen.getByRole("button", { name: "Start" }));

    expect(onStart).toHaveBeenCalledTimes(1);
    expect(onStart).toHaveBeenCalledWith("ThirtyTwo");
  });
});
