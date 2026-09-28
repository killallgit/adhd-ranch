import { listen } from "@tauri-apps/api/event";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { act } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import type { FocusWriter } from "../api/focusWriter";
import type { RenderSceneReader } from "../api/tauriRenderSceneReader";
import type { Focus } from "../types/focus";
import type { Animal } from "../types/generated/Animal";
import type { RenderScene } from "../types/generated/RenderScene";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const movement = vi.hoisted(() => ({ frozenId: null as string | null }));

vi.mock(import("../hooks/useAnimalMovement"), async () => {
  return {
    useAnimalMovement: (scene: RenderScene, _displaySpace: unknown, frozenId: string | null) => {
      movement.frozenId = frozenId;
      return {
        animals: scene.animals.map((animal) => ({
          id: animal.id,
          label: animal.label,
          x: 100,
          y: 100,
          vx: 1,
          vy: 0,
          frameIndex: 0,
          direction: "right" as const,
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

const PROFILE = {
  species: "pig" as const,
  baseSizePx: 48,
  movementFootprintPx: 48,
  walkSpeedPxPerSecond: 60,
  friction: 0.97,
  minimumSpeedFraction: 0.35,
  maximumTossSpeedMultiplier: 6,
  turnMinMs: 3_000,
  turnMaxMs: 8_000,
  frameIntervalMs: 150,
};

const SAMPLE_FOCUSES: Focus[] = [
  { id: "a", title: "Customer X bug", description: "", created_at: "", tasks: [] },
  { id: "b", title: "API refactor", description: "", created_at: "", tasks: [] },
];

function animal(overrides: Partial<Animal> = {}): Animal {
  return {
    id: "a",
    label: "Customer X bug",
    species: "pig",
    size: { kind: "fixed", px: 48 },
    regionId: null,
    motion: "walking",
    ...overrides,
  };
}

function sceneReader(animals: readonly Animal[]): RenderSceneReader {
  const scene: RenderScene = {
    animals: [...animals],
    regions: [],
    speciesProfiles: [PROFILE],
  };
  return {
    read: vi.fn().mockResolvedValue(scene),
    subscribe: vi.fn().mockResolvedValue(() => {}),
  };
}

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

function renderApp(
  focuses: readonly Focus[],
  animals: readonly Animal[],
  writer: FocusWriter = noopFocusWriter(),
) {
  return render(
    <App
      focusReader={createFixtureFocusReader(focuses)}
      focusWriter={writer}
      onWriteFailure={() => {}}
      renderSceneReader={sceneReader(animals)}
    />,
  );
}

beforeEach(() => {
  movement.frozenId = null;
  vi.mocked(listen).mockResolvedValue(() => {});
});

describe("App overlay", () => {
  it("renders the overlay root", async () => {
    renderApp([], []);
    expect(document.querySelector(".overlay-root")).toBeInTheDocument();
    await act(async () => {});
  });

  it("renders mixed generated-scene Animals through explicit Species dispatch", async () => {
    renderApp(SAMPLE_FOCUSES, [
      animal({ label: "Focus from scene" }),
      animal({
        id: "agent:one",
        label: "Agent from scene",
        regionId: "repo",
        motion: "resting",
      }),
    ]);

    expect(await screen.findByText("Focus from scene")).toBeInTheDocument();
    expect(screen.getByText("Agent from scene")).toBeInTheDocument();
  });

  it("keeps the Focus reader for detail and edit data", async () => {
    const writer = noopFocusWriter();
    renderApp(SAMPLE_FOCUSES, [animal()], writer);

    await userEvent.click(await screen.findByText("Customer X bug"));
    await userEvent.type(screen.getByPlaceholderText("Add task…"), "write tests{Enter}");

    expect(writer.appendTask).toHaveBeenCalledWith("a", "write tests");
    expect(await screen.findByLabelText("task text: write tests")).toHaveValue("write tests");
  });

  it("samples absolute size and renders resting Motion", async () => {
    renderApp(
      [],
      [
        animal({
          id: "expired",
          label: "Expired",
          size: { kind: "fixed", px: 96 },
          motion: "resting",
        }),
      ],
    );

    const sprite = await screen.findByRole("button", { name: /expired/i });
    expect(sprite).toHaveClass("pig-sprite--resting");
    expect(sprite.querySelector(".pig-sprite-frame")).toHaveStyle({
      width: "96px",
      height: "96px",
    });
  });

  it("opens a current Focus when the tray requests its render id", async () => {
    const listeners = new Map<string, (event: { payload: string }) => void>();
    vi.mocked(listen).mockImplementation((event, callback) => {
      listeners.set(event, callback as (event: { payload: string }) => void);
      return Promise.resolve(() => {});
    });
    renderApp(SAMPLE_FOCUSES, [animal({ id: "b", label: "API refactor" })]);
    await screen.findByText("API refactor");

    act(() => listeners.get("open-focus-detail")?.({ payload: "b" }));

    expect(screen.getByLabelText("focus title")).toHaveValue("API refactor");
    expect(movement.frozenId).toBe("b");
  });

  it("does not open Focus details for an Agent id", async () => {
    renderApp(SAMPLE_FOCUSES, [animal({ id: "agent:one", label: "Agent" })]);
    await userEvent.click(await screen.findByText("Agent"));

    expect(screen.queryByPlaceholderText("Add task…")).not.toBeInTheDocument();
    expect(movement.frozenId).toBeNull();
  });

  it("clears detail and frozen id when the selected Focus disappears", async () => {
    const { rerender } = renderApp(SAMPLE_FOCUSES, [animal()]);
    await userEvent.click(await screen.findByText("Customer X bug"));
    expect(screen.getByLabelText("focus title")).toHaveValue("Customer X bug");

    rerender(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        renderSceneReader={sceneReader([])}
      />,
    );

    await waitFor(() => expect(screen.queryByLabelText("focus title")).not.toBeInTheDocument());
    expect(movement.frozenId).toBeNull();
  });

  it("updates Focus size/Motion and Agent Motion independently", async () => {
    let current: RenderScene = {
      animals: [animal({ id: "a", label: "Focus" }), animal({ id: "agent:one", label: "Agent" })],
      regions: [],
      speciesProfiles: [PROFILE],
    };
    let invalidate: (() => void) | null = null;
    const reader: RenderSceneReader = {
      read: vi.fn().mockImplementation(async () => current),
      subscribe: vi.fn().mockImplementation(async (callback) => {
        invalidate = () => callback({ event: "focuses-changed" });
        return () => {};
      }),
    };
    render(
      <App
        focusReader={createFixtureFocusReader(SAMPLE_FOCUSES)}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        renderSceneReader={reader}
      />,
    );
    const focusSprite = await screen.findByRole("button", { name: /focus/i });
    const agentSprite = screen.getByRole("button", { name: /agent/i });

    current = {
      ...current,
      animals: [
        animal({ id: "a", label: "Focus", size: { kind: "fixed", px: 144 }, motion: "resting" }),
        animal({ id: "agent:one", label: "Agent" }),
      ],
    };
    await act(async () => invalidate?.());
    await waitFor(() => expect(focusSprite).toHaveClass("pig-sprite--resting"));
    expect(focusSprite.querySelector(".pig-sprite-frame")).toHaveStyle({ width: "144px" });
    expect(agentSprite).not.toHaveClass("pig-sprite--resting");

    current = {
      ...current,
      animals: [current.animals[0], animal({ id: "agent:one", label: "Agent", motion: "resting" })],
    };
    await act(async () => invalidate?.());
    await waitFor(() => expect(agentSprite).toHaveClass("pig-sprite--resting"));
    expect(focusSprite.querySelector(".pig-sprite-frame")).toHaveStyle({ width: "144px" });
  });

  it("renders scene regions without Pen data", async () => {
    const reader: RenderSceneReader = {
      read: vi.fn().mockResolvedValue({
        animals: [],
        regions: [
          {
            id: "repo",
            label: "Repository",
            rect: { x: 40, y: 50, w: 320, h: 240 },
            hue: 145,
          },
        ],
        speciesProfiles: [PROFILE],
      } satisfies RenderScene),
      subscribe: vi.fn().mockResolvedValue(() => {}),
    };
    render(
      <App
        focusReader={createFixtureFocusReader([])}
        focusWriter={noopFocusWriter()}
        onWriteFailure={() => {}}
        renderSceneReader={reader}
      />,
    );

    expect(await screen.findByText("Repository")).toBeInTheDocument();
    expect(document.querySelector(".region-box")).toHaveStyle({
      left: "40px",
      top: "50px",
      width: "320px",
      height: "240px",
    });
  });
});
