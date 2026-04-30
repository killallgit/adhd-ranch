import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";
import { PendingTray } from "./PendingTray";

const focuses: Focus[] = [{ id: "f1", title: "Customer X bug", description: "", tasks: [] }];

const addTaskProposal: Proposal = {
  id: "p1",
  kind: "add_task",
  target_focus_id: "f1",
  task_text: "ship it",
  summary: "did a thing",
  reasoning: "fits",
  created_at: "2026-04-30T12:00:00Z",
};

describe("PendingTray", () => {
  it("renders nothing when there are no proposals", () => {
    const { container } = render(<PendingTray proposals={[]} focuses={focuses} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("shows badge count when collapsed", () => {
    render(<PendingTray proposals={[addTaskProposal]} focuses={focuses} />);
    expect(screen.getByTestId("pending-tray-count")).toHaveTextContent("1");
    expect(screen.queryByTestId("proposal-card")).not.toBeInTheDocument();
  });

  it("expands to show proposals with target focus title", () => {
    render(<PendingTray proposals={[addTaskProposal]} focuses={focuses} />);
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    expect(screen.getByTestId("proposal-card")).toBeInTheDocument();
    expect(screen.getByTestId("proposal-target")).toHaveTextContent('Add to "Customer X bug"');
    expect(screen.queryByTestId("proposal-reasoning")).not.toBeInTheDocument();
  });

  it("reveals reasoning when ? is toggled", () => {
    render(<PendingTray proposals={[addTaskProposal]} focuses={focuses} />);
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    fireEvent.click(screen.getByLabelText(/toggle reasoning/));
    expect(screen.getByTestId("proposal-reasoning")).toHaveTextContent("fits");
  });
});
