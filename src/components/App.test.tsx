import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import { createFixtureProposalReader } from "../api/fixtureProposalReader";
import type { Focus } from "../types/focus";
import { App } from "./App";

const sample: Focus[] = [{ id: "a", title: "Customer X bug", description: "", tasks: [] }];

describe("App", () => {
  it("renders root element and shows focuses once ready", async () => {
    const focusReader = createFixtureFocusReader(sample);
    const proposalReader = createFixtureProposalReader([]);
    render(<App focusReader={focusReader} proposalReader={proposalReader} />);
    expect(screen.getByTestId("app-root")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText("Customer X bug")).toBeInTheDocument();
    });
  });
});
