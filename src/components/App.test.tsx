import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import { createFixtureProposalReader } from "../api/fixtureProposalReader";
import type { FocusWriter } from "../api/focusWriter";
import type { ProposalWriter } from "../api/proposals";
import type { Focus } from "../types/focus";
import { App } from "./App";

const sample: Focus[] = [{ id: "a", title: "Customer X bug", description: "", tasks: [] }];

const noopProposalWriter: ProposalWriter = {
  accept: () => Promise.resolve({ id: "x", target: null }),
  reject: () => Promise.resolve({ id: "x", target: null }),
};

function noopFocusWriter(): FocusWriter {
  return {
    createFocus: vi.fn().mockResolvedValue({ id: "any" }),
    deleteFocus: vi.fn().mockResolvedValue(undefined),
    appendTask: vi.fn().mockResolvedValue(undefined),
    deleteTask: vi.fn().mockResolvedValue(undefined),
  };
}

describe("App", () => {
  it("renders root element and shows focuses once ready", async () => {
    const focusReader = createFixtureFocusReader(sample);
    const proposalReader = createFixtureProposalReader([]);
    render(
      <App
        focusReader={focusReader}
        focusWriter={noopFocusWriter()}
        proposalReader={proposalReader}
        proposalWriter={noopProposalWriter}
      />,
    );
    expect(screen.getByTestId("app-root")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText("Customer X bug")).toBeInTheDocument();
    });
  });
});
