import { listen } from "@tauri-apps/api/event";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { act } from "react";
import { describe, expect, it, vi } from "vitest";
import { createFixtureAgentSessionReader } from "../api/fixtureAgentSessionReader";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import { createFixtureSettingsReader } from "../api/fixtureSettingsReader";
import type { FocusWriter } from "../api/focusWriter";
import type { AgentSession } from "../types/agentSession";
import type { Animal } from "../types/animal";
import type { Focus } from "../types/focus";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

// The overlay widens its hit rect to the whole viewport while an Animal is
// selected, so what App passes as selectedId decides whether clicks reach the
// desktop behind it.
const movement = vi.hoisted(() => ({ selectedId: null as string | null }));

const PEN_AREA = { x: 0, y: 0, w: 1000, h: 800 };

vi.mock(import("../hooks/usePigMovement"), async (importOriginal) => {
  const actual = await importOriginal();
  const { animalPen } = await import("../lib/animals");
  const { layoutPens, uniquePens } = await import("../lib/session/pens");
  return {
    ...actual,
    usePigMovement: (
      animals: readonly Animal[],
      selectedId: string | null,
      maxPenSize: number | null,
    ) => {
      movement.selectedId = selectedId;
      return {
        pens:
          maxPenSize === null
            ? []
            : layoutPens(uniquePens(animals.map(animalPen)), PEN_AREA, maxPenSize),
        pigs: animals.map((animal) => ({
          id: animal.id,
          name: animal.name,
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
      };
    },
  };
});

const noAgents = createFixtureAgentSessionReader([]);
const defaultSettings = createFixtureSettingsReader();

function session(
  name: string,
  checkout: string,
  activity: AgentSession["activity"] = "Idle",
): AgentSession {
  return {
    id: `session-${name}`,
    name,
    pen: { id: `/code/${checkout}`, name: checkout },
    activity,
  };
}

const sample: Focus[] = [
  { id: "a", title: "Customer X bug", description: "", created_at: "", tasks: [] },
  { id: "b", title: "API refactor", description: "", created_at: "", tasks: [] },
];

function noopFocusWriter(): FocusWriter {
  const ok = { ok: true } as const;
  return {
    createFocus: vi.fn().mockResolvedValue(ok),
    duplicateFocus: vi.fn().mockResolvedValue(ok),
    deleteFocus: vi.fn().mockResolvedValue(ok),
    renameFocus: vi.fn().mockResolvedValue(ok),
    appendTask: vi.fn().mockResolvedValue(ok),
    deleteTask: vi.fn().mockResolvedValue(ok),
    updateTask: vi.fn().mockResolvedValue(ok),
    toggleTask: vi.fn().mockResolvedValue(ok),
    startTimer: vi.fn().mockResolvedValue(ok),
    clearTimer: vi.fn().mockResolvedValue(ok),
  };
}

describe("App overlay", () => {
  it("renders the overlay root", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );
    expect(document.querySelector(".overlay-root")).toBeInTheDocument();
    await act(async () => {});
  });

  it("spawns a pig for each focus", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
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
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Customer X bug")).toBeInTheDocument();
    });

    // Click the pig to open AnimalDetail
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
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );

    await screen.findByText("Customer X bug");
    await userEvent.click(screen.getByText("Customer X bug"));

    await userEvent.type(screen.getByPlaceholderText("Add task…"), "write tests{Enter}");

    expect(await screen.findByLabelText("task text: write tests")).toHaveValue("write tests");
  });

  it("renders a timed focus with the current animal scale", async () => {
    const nowSpy = vi.spyOn(Date, "now").mockReturnValue(1_060_000);
    try {
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
          agentSessionReader={noAgents}
          settingsReader={defaultSettings}
        />,
      );

      const pig = await screen.findByRole("button", { name: /timer focus/i });
      expect(pig.querySelector(".pig-sprite-frame")).toHaveStyle({ width: "96px", height: "96px" });
    } finally {
      nowSpy.mockRestore();
    }
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
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );

    const pig = await screen.findByRole("button", { name: /expired focus/i });
    expect(pig).toHaveClass("pig-sprite--resting");
    expect(pig.querySelector(".pig-sprite-frame")).toHaveStyle({
      filter:
        "grayscale(1) saturate(0.15) brightness(1.55) drop-shadow(0 0 8px rgba(210, 240, 255, 0.55))",
      opacity: "0.48",
    });
  });

  it("does not render expired animal visual for an expired task timer", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([
          {
            id: "task-expired",
            title: "Task expired focus",
            description: "",
            created_at: "",
            tasks: [
              {
                id: "task-1",
                text: "Write tests",
                done: false,
                timer: { duration_secs: 120, started_at: 1_000, status: "Expired" },
              },
            ],
            timer: null,
          },
        ])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );

    const pig = await screen.findByRole("button", { name: /task expired focus/i });
    expect(pig).not.toHaveClass("pig-sprite--resting");
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
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );

    await screen.findByText("API refactor");
    await act(async () => {
      listeners.get("open-focus-detail")?.({ payload: "b" });
    });

    expect(screen.getByLabelText("focus title")).toHaveValue("API refactor");
  });

  it("spawns agent pigs alongside focus pigs", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={createFixtureAgentSessionReader([session("feature-abc", "adhd-ranch")])}
        settingsReader={defaultSettings}
      />,
    );

    expect(await screen.findByText("feature-abc")).toBeInTheDocument();
    expect(screen.getByText("Customer X bug")).toBeInTheDocument();
    expect(screen.getByText("API refactor")).toBeInTheDocument();
  });

  it("draws one pen per checkout", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={createFixtureAgentSessionReader([
          session("feature-abc", "adhd-ranch"),
          session("main", "adhd-ranch"),
          session("spike", "other-repo"),
        ])}
        settingsReader={defaultSettings}
      />,
    );

    expect(await screen.findByText("adhd-ranch")).toBeInTheDocument();
    expect(screen.getByText("other-repo")).toBeInTheDocument();
    expect(document.querySelectorAll(".pen-box")).toHaveLength(2);
  });

  it("tells two pens apart by colour", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={createFixtureAgentSessionReader([
          session("feature-abc", "adhd-ranch"),
          session("spike", "other-repo"),
        ])}
        settingsReader={defaultSettings}
      />,
    );

    await screen.findByText("adhd-ranch");
    const hues = [...document.querySelectorAll<HTMLElement>(".pen-box")].map((pen) =>
      pen.style.getPropertyValue("--pen-hue"),
    );

    expect(hues[0]).not.toBe("");
    expect(hues[0]).not.toBe(hues[1]);
  });

  it("draws a pen's border around the cell its animals roam", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={createFixtureAgentSessionReader([session("feature-abc", "adhd-ranch")])}
        settingsReader={defaultSettings}
      />,
    );

    await screen.findByText("feature-abc");
    const pen = document.querySelector(".pen-box");

    expect(pen).toHaveStyle({ left: "340px", top: "240px", width: "320px", height: "320px" });
  });

  it("draws a pen no larger than the settings allow", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={createFixtureAgentSessionReader([session("feature-abc", "adhd-ranch")])}
        settingsReader={createFixtureSettingsReader({ pens: { max_size: 200 } })}
      />,
    );

    await waitFor(() =>
      expect(document.querySelector(".pen-box")).toHaveStyle({
        width: "200px",
        height: "200px",
      }),
    );
  });

  it("rests a session between turns as a ghost", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={createFixtureAgentSessionReader([session("feature-abc", "adhd-ranch")])}
        settingsReader={defaultSettings}
      />,
    );

    const pig = await screen.findByRole("button", { name: /feature-abc/i });

    expect(pig).toHaveClass("pig-sprite--resting");
  });

  it("keeps a working session's animal awake", async () => {
    const nowSpy = vi.spyOn(Date, "now").mockReturnValue(1_000_000_000);
    try {
      render(
        <App
          focusReader={createFixtureFocusReader([])}
          focusWriter={noopFocusWriter()}
          onWriteFailure={() => {}}
          agentSessionReader={createFixtureAgentSessionReader([
            session("feature-abc", "adhd-ranch", "Working"),
          ])}
          settingsReader={defaultSettings}
        />,
      );

      const pig = await screen.findByRole("button", { name: /feature-abc/i });

      expect(pig).not.toHaveClass("pig-sprite--resting");
    } finally {
      nowSpy.mockRestore();
    }
  });

  it("closes animal detail when the selected focus disappears", async () => {
    // A selection that outlives its Focus used to keep the overlay's full-viewport
    // hit rect, swallowing every click on the desktop behind it.
    const { rerender } = render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );

    await userEvent.click(await screen.findByText("Customer X bug"));
    expect(screen.getByLabelText("focus title")).toHaveValue("Customer X bug");

    rerender(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={noAgents}
        settingsReader={defaultSettings}
      />,
    );

    await waitFor(() => expect(screen.queryByLabelText("focus title")).not.toBeInTheDocument());
    expect(movement.selectedId).toBeNull();
  });

  it("does not open animal detail when an agent pig is clicked", async () => {
    render(
      <App
        focusReader={createFixtureFocusReader(sample)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        agentSessionReader={createFixtureAgentSessionReader([session("feature-abc", "adhd-ranch")])}
        settingsReader={defaultSettings}
      />,
    );

    await userEvent.click(await screen.findByText("feature-abc"));

    expect(screen.queryByPlaceholderText("Add task…")).not.toBeInTheDocument();
  });
});
