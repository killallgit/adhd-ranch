import { listen } from "@tauri-apps/api/event";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { act } from "react";
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
  { id: "a", title: "Customer X bug", description: "", created_at: "", tasks: [] },
  { id: "b", title: "API refactor", description: "", created_at: "", tasks: [] },
];

function noopFocusWriter(): FocusWriter {
  const ok = { ok: true } as const;
  return {
    createFocus: vi.fn().mockResolvedValue(ok),
    deleteFocus: vi.fn().mockResolvedValue(ok),
    renameFocus: vi.fn().mockResolvedValue(ok),
    appendTask: vi.fn().mockResolvedValue(ok),
    deleteTask: vi.fn().mockResolvedValue(ok),
    updateTask: vi.fn().mockResolvedValue(ok),
    toggleTask: vi.fn().mockResolvedValue(ok),
    startTimer: vi.fn().mockResolvedValue(ok),
  };
}

describe("App overlay", () => {
  it("renders the overlay root", () => {
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
      />,
    );
    expect(document.querySelector(".overlay-root")).toBeInTheDocument();
  });

  it("spawns a pig for each focus", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
      />,
    );
    await waitFor(() => {
      expect(screen.getByText("Customer X bug")).toBeInTheDocument();
      expect(screen.getByText("API refactor")).toBeInTheDocument();
    });
  });

  it("add-task input calls focusWriter.appendTask with selected focus id", async () => {
    const writer = noopFocusWriter();
    render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={writer}
        onWriteFailure={() => {}}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Customer X bug")).toBeInTheDocument();
    });

    // Click the pig to open PigDetail
    await userEvent.click(screen.getByText("Customer X bug"));

    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "write tests{Enter}");

    expect(writer.appendTask).toHaveBeenCalledWith("a", "write tests");
  });

  it("shows an added task in the open detail card after append succeeds", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
      />,
    );

    await screen.findByText("Customer X bug");
    await userEvent.click(screen.getByText("Customer X bug"));

    await userEvent.type(screen.getByPlaceholderText("Add task…"), "write tests{Enter}");

    expect(await screen.findByLabelText("task text: write tests")).toHaveValue("write tests");
  });

  it("renders a timed focus with the current animal scale", async () => {
    const nowSpy = vi.spyOn(Date, "now").mockReturnValue(1_060_000);
    render(
      <App
        focusReader={createFixtureFocusReader([
          {
            id: "timed",
            title: "Timer focus",
            description: "",
            created_at: "",
            tasks: [],
            timer: { duration_secs: 120, started_at: 1_000, status: "Running" },
          },
        ])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
      />,
    );

    const pig = await screen.findByRole("button", { name: /timer focus/i });
    expect(pig.querySelector(".pig-sprite-frame")).toHaveStyle({ width: "96px", height: "96px" });
    nowSpy.mockRestore();
  });

  it("renders an expired focus with an expired animal visual", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([
          {
            id: "expired",
            title: "Expired focus",
            description: "",
            created_at: "",
            tasks: [],
            timer: { duration_secs: 120, started_at: 1_000, status: "Expired" },
          },
        ])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
      />,
    );

    const pig = await screen.findByRole("button", { name: /expired focus/i });
    expect(pig).toHaveClass("pig-sprite--expired");
    expect(pig.querySelector(".pig-sprite-frame")).toHaveStyle({
      filter:
        "grayscale(1) saturate(0.15) brightness(1.55) drop-shadow(0 0 8px rgba(210, 240, 255, 0.55))",
      opacity: "0.48",
    });
  });

  it("opens animal detail when the tray asks to open a focus", async () => {
    const listeners = new Map<string, (event: { payload: string }) => void>();
    vi.mocked(listen).mockImplementation((event, cb) => {
      listeners.set(event, cb as (event: { payload: string }) => void);
      return Promise.resolve(() => {});
    });

    render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
      />,
    );

    await screen.findByText("API refactor");
    await act(async () => {
      listeners.get("open-focus-detail")?.({ payload: "b" });
    });

    expect(screen.getByLabelText("focus title")).toHaveValue("API refactor");
  });
});
