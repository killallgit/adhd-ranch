import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import type { FocusWriter } from "../api/focusWriter";
import type { Focus } from "../types/focus";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock(import("../hooks/usePigMovement"), async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    usePigMovement: (focuses: readonly Focus[]) =>
      focuses.map((f) => ({
        id: f.id,
        name: f.title,
        x: 100,
        y: 100,
        vx: 1,
        vy: 0,
        frameIndex: 0,
        direction: "right" as "left" | "right",
        lastFrameAt: 0,
        nextTurnAt: 9_999_999,
      })),
  };
});

const sample: Focus[] = [
  { id: "a", title: "Customer X bug", description: "", tasks: [] },
  { id: "b", title: "API refactor", description: "", tasks: [] },
];

function noopFocusWriter(): FocusWriter {
  return {
    createFocus: vi.fn().mockResolvedValue({ id: "any" }),
    deleteFocus: vi.fn().mockResolvedValue(undefined),
    appendTask: vi.fn().mockResolvedValue(undefined),
    deleteTask: vi.fn().mockResolvedValue(undefined),
  };
}

describe("App overlay", () => {
  it("renders the overlay root", () => {
    render(<App focusReader={createFixtureFocusReader([])} focusWriter={noopFocusWriter()} />);
    expect(document.querySelector(".overlay-root")).toBeInTheDocument();
  });

  it("spawns a pig for each focus", async () => {
    render(<App focusReader={createFixtureFocusReader(sample)} focusWriter={noopFocusWriter()} />);
    await waitFor(() => {
      expect(screen.getByText("Customer X bug")).toBeInTheDocument();
      expect(screen.getByText("API refactor")).toBeInTheDocument();
    });
  });
});
