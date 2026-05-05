import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Focus } from "../types/focus";
import { FocusCard } from "./FocusCard";

const focus: Focus = {
  id: "f1",
  title: "Customer X bug",
  description: "",
  tasks: [{ id: "t1", text: "ship", done: false }],
};

describe("FocusCard", () => {
  it("requires a confirm step before delete", () => {
    const onDelete = vi.fn();
    render(<FocusCard focus={focus} busy={false} onClearTask={vi.fn()} onDeleteFocus={onDelete} />);
    expect(screen.queryByTestId("focus-delete-confirm")).not.toBeInTheDocument();
    fireEvent.click(screen.getByLabelText("menu Customer X bug"));
    expect(screen.getByTestId("focus-delete-confirm")).toBeInTheDocument();
    expect(onDelete).not.toHaveBeenCalled();

    fireEvent.click(screen.getByText("Delete"));
    expect(onDelete).toHaveBeenCalledWith("f1");
  });

  it("cancel hides the confirm without calling onDelete", () => {
    const onDelete = vi.fn();
    render(<FocusCard focus={focus} busy={false} onClearTask={vi.fn()} onDeleteFocus={onDelete} />);
    fireEvent.click(screen.getByLabelText("menu Customer X bug"));
    fireEvent.click(screen.getByText("Cancel"));
    expect(screen.queryByTestId("focus-delete-confirm")).not.toBeInTheDocument();
    expect(onDelete).not.toHaveBeenCalled();
  });
});
