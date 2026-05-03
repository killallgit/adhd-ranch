import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Focus } from "../types/focus";
import { PigDetail } from "./PigDetail";

const baseFocus: Focus = {
  id: "pig-a",
  title: "Ship it",
  description: "",
  tasks: [],
};

function renderDetail(overrides?: Partial<React.ComponentProps<typeof PigDetail>>) {
  const props = {
    focus: baseFocus,
    pigX: 100,
    pigY: 100,
    viewportW: 1920,
    viewportH: 1080,
    onClose: vi.fn(),
    onClearTask: vi.fn(),
    onAddTask: vi.fn(),
    ...overrides,
  };
  render(<PigDetail {...props} />);
  return props;
}

describe("PigDetail add-task input", () => {
  it("renders an add task input with placeholder", () => {
    renderDetail();
    expect(screen.getByPlaceholderText("Add task…")).toBeInTheDocument();
  });

  it("Enter calls onAddTask with trimmed text", async () => {
    const { onAddTask } = renderDetail();
    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "fix the thing{Enter}");
    expect(onAddTask).toHaveBeenCalledWith("fix the thing");
  });

  it("Enter clears the input field", async () => {
    renderDetail();
    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "some task{Enter}");
    expect(input).toHaveValue("");
  });

  it("Enter with empty text does not call onAddTask", async () => {
    const { onAddTask } = renderDetail();
    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "   {Enter}");
    expect(onAddTask).not.toHaveBeenCalled();
  });

  it("Escape closes the card", async () => {
    const { onClose } = renderDetail();
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();
  });
});
