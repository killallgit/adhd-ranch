import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Focus } from "../types/focus";
import { FocusList } from "./FocusList";

const sample: Focus[] = [
  {
    id: "a",
    title: "Customer X bug",
    description: "",
    tasks: [
      { id: "t1", text: "ship the fix", done: false },
      { id: "t2", text: "verify on staging", done: false },
    ],
  },
  {
    id: "b",
    title: "API refactor",
    description: "",
    tasks: [{ id: "t3", text: "extract pipeline", done: false }],
  },
];

describe("FocusList", () => {
  it("renders one card per focus and one row per task", () => {
    render(
      <FocusList
        focuses={sample}
        busyFocusId={null}
        onClearTask={vi.fn()}
        onDeleteFocus={vi.fn()}
      />,
    );
    expect(screen.getAllByTestId("focus-card")).toHaveLength(2);
    expect(screen.getAllByTestId("task-row")).toHaveLength(3);
  });

  it("renders empty-state when no focuses", () => {
    render(
      <FocusList focuses={[]} busyFocusId={null} onClearTask={vi.fn()} onDeleteFocus={vi.fn()} />,
    );
    expect(screen.getByTestId("focus-list-empty")).toBeInTheDocument();
  });

  it("invokes onClearTask with focus id and zero-based index", () => {
    const onClear = vi.fn();
    render(
      <FocusList
        focuses={sample}
        busyFocusId={null}
        onClearTask={onClear}
        onDeleteFocus={vi.fn()}
      />,
    );
    fireEvent.click(screen.getAllByLabelText(/clear task/)[1]);
    expect(onClear).toHaveBeenCalledWith("a", 1);
  });
});
