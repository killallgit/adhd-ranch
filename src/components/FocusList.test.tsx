import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Focus } from "../types/focus";
import { FocusList } from "./FocusList";

const sample: Focus[] = [
  {
    id: "a",
    title: "Customer X bug",
    description: "",
    tasks: [
      { id: "t1", text: "ship the fix" },
      { id: "t2", text: "verify on staging" },
    ],
  },
  {
    id: "b",
    title: "API refactor",
    description: "",
    tasks: [{ id: "t3", text: "extract pipeline" }],
  },
];

describe("FocusList", () => {
  it("renders one card per focus and one row per task", () => {
    render(<FocusList focuses={sample} />);
    expect(screen.getAllByTestId("focus-card")).toHaveLength(2);
    expect(screen.getAllByTestId("task-row")).toHaveLength(3);
    expect(screen.getByText("Customer X bug")).toBeInTheDocument();
    expect(screen.getByText("ship the fix")).toBeInTheDocument();
  });

  it("renders empty-state hero when no focuses", () => {
    render(<FocusList focuses={[]} />);
    expect(screen.getByTestId("focus-list-empty")).toBeInTheDocument();
  });
});
