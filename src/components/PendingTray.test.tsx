import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ProposalWriter } from "../api/proposals";
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

const noopWriter: ProposalWriter = {
  accept: () => Promise.resolve({ id: "x", target: null }),
  reject: () => Promise.resolve({ id: "x", target: null }),
};

describe("PendingTray", () => {
  it("renders nothing when there are no proposals", () => {
    const { container } = render(
      <PendingTray proposals={[]} focuses={focuses} proposalWriter={noopWriter} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("shows badge count when collapsed", () => {
    render(
      <PendingTray proposals={[addTaskProposal]} focuses={focuses} proposalWriter={noopWriter} />,
    );
    expect(screen.getByTestId("pending-tray-count")).toHaveTextContent("1");
    expect(screen.queryByTestId("proposal-card")).not.toBeInTheDocument();
  });

  it("expands to show proposals with target focus title", () => {
    render(
      <PendingTray proposals={[addTaskProposal]} focuses={focuses} proposalWriter={noopWriter} />,
    );
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    expect(screen.getByTestId("proposal-card")).toBeInTheDocument();
    expect(screen.getByTestId("proposal-target")).toHaveTextContent('Add to "Customer X bug"');
  });

  it("calls writer.accept when the ✓ button is clicked", async () => {
    const writer: ProposalWriter = {
      accept: vi.fn().mockResolvedValue({ id: "p1", target: "f1" }),
      reject: vi.fn().mockResolvedValue({ id: "p1", target: null }),
    };
    render(<PendingTray proposals={[addTaskProposal]} focuses={focuses} proposalWriter={writer} />);
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    fireEvent.click(screen.getByLabelText(/accept proposal p1/));
    await waitFor(() => {
      expect(writer.accept).toHaveBeenCalledWith("p1", undefined);
    });
  });

  it("calls writer.reject when the ✗ button is clicked", async () => {
    const writer: ProposalWriter = {
      accept: vi.fn().mockResolvedValue({ id: "p1", target: null }),
      reject: vi.fn().mockResolvedValue({ id: "p1", target: null }),
    };
    render(<PendingTray proposals={[addTaskProposal]} focuses={focuses} proposalWriter={writer} />);
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    fireEvent.click(screen.getByLabelText(/reject proposal p1/));
    await waitFor(() => {
      expect(writer.reject).toHaveBeenCalledWith("p1");
    });
  });

  it("opens the edit modal and forwards overrides on accept", async () => {
    const writer: ProposalWriter = {
      accept: vi.fn().mockResolvedValue({ id: "p1", target: "f1" }),
      reject: vi.fn().mockResolvedValue({ id: "p1", target: null }),
    };
    render(<PendingTray proposals={[addTaskProposal]} focuses={focuses} proposalWriter={writer} />);
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    fireEvent.click(screen.getByLabelText(/edit proposal p1/));
    fireEvent.change(screen.getByLabelText("task text"), {
      target: { value: "ship it (edited)" },
    });
    fireEvent.click(screen.getByText("Accept (edited)"));
    await waitFor(() => {
      expect(writer.accept).toHaveBeenCalledWith("p1", {
        target_focus_id: "f1",
        task_text: "ship it (edited)",
      });
    });
  });

  it("surfaces writer errors inline", async () => {
    const writer: ProposalWriter = {
      accept: vi.fn().mockRejectedValue(new Error("focus not found")),
      reject: vi.fn().mockResolvedValue({ id: "p1", target: null }),
    };
    render(<PendingTray proposals={[addTaskProposal]} focuses={focuses} proposalWriter={writer} />);
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    fireEvent.click(screen.getByLabelText(/accept proposal p1/));
    await waitFor(() => {
      expect(screen.getByTestId("proposal-error")).toHaveTextContent("focus not found");
    });
  });
});
