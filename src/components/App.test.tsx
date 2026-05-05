import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import type { FocusWriter } from "../api/focusWriter";
import type { Focus } from "../types/focus";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock(import("../hooks/usePigMovement"), async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    usePigMovement: (focuses: readonly Focus[]) => ({
      pigs: focuses.map((f) => ({
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
      startDrag: vi.fn(),
      moveDrag: vi.fn(),
      endDrag: vi.fn(() => ({ wasDrag: false })),
      setDragActive: vi.fn(),
    }),
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
    renameFocus: vi.fn().mockResolvedValue(undefined),
    appendTask: vi.fn().mockResolvedValue(undefined),
    deleteTask: vi.fn().mockResolvedValue(undefined),
    updateTask: vi.fn().mockResolvedValue(undefined),
    toggleTask: vi.fn().mockResolvedValue(undefined),
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

  it("add-task input calls focusWriter.appendTask with selected focus id", async () => {
    const writer = noopFocusWriter();
    render(<App focusReader={createFixtureFocusReader(sample)} focusWriter={writer} />);

    await waitFor(() => {
      expect(screen.getByText("Customer X bug")).toBeInTheDocument();
    });

    // Click the pig to open PigDetail
    await userEvent.click(screen.getByText("Customer X bug"));

    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "write tests{Enter}");

    expect(writer.appendTask).toHaveBeenCalledWith("a", "write tests");
  });
});
