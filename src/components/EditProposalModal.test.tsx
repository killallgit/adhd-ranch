import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";
import { EditProposalModal } from "./EditProposalModal";

const focuses: Focus[] = [
  { id: "f1", title: "Customer X bug", description: "", tasks: [] },
  { id: "f2", title: "API refactor", description: "", tasks: [] },
];

const addTaskProposal: Proposal = {
  id: "p1",
  kind: "add_task",
  target_focus_id: "f1",
  task_text: "ship it",
  summary: "did a thing",
  reasoning: "fits",
  created_at: "2026-04-30T12:00:00Z",
};

const newFocusProposal: Proposal = {
  id: "p2",
  kind: "new_focus",
  new_focus: { title: "Spike auth", description: "look at OIDC" },
  summary: "did a thing",
  reasoning: "doesn't fit",
  created_at: "2026-04-30T12:00:00Z",
};

describe("EditProposalModal", () => {
  it("submits add_task overrides on confirm", () => {
    const onConfirm = vi.fn();
    render(
      <EditProposalModal
        proposal={addTaskProposal}
        focuses={focuses}
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />,
    );
    fireEvent.change(screen.getByLabelText("target focus"), { target: { value: "f2" } });
    fireEvent.change(screen.getByLabelText("task text"), {
      target: { value: "ship it (edited)" },
    });
    fireEvent.click(screen.getByText("Accept (edited)"));
    expect(onConfirm).toHaveBeenCalledWith({
      target_focus_id: "f2",
      task_text: "ship it (edited)",
    });
  });

  it("submits new_focus overrides on confirm", () => {
    const onConfirm = vi.fn();
    render(
      <EditProposalModal
        proposal={newFocusProposal}
        focuses={focuses}
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />,
    );
    fireEvent.change(screen.getByLabelText("new focus title"), {
      target: { value: "OIDC spike" },
    });
    fireEvent.click(screen.getByText("Accept (edited)"));
    expect(onConfirm).toHaveBeenCalledWith({
      new_focus: { title: "OIDC spike", description: "look at OIDC" },
    });
  });

  it("calls onCancel when cancel clicked", () => {
    const onCancel = vi.fn();
    render(
      <EditProposalModal
        proposal={addTaskProposal}
        focuses={focuses}
        onConfirm={vi.fn()}
        onCancel={onCancel}
      />,
    );
    fireEvent.click(screen.getByText("Cancel"));
    expect(onCancel).toHaveBeenCalled();
  });
});
