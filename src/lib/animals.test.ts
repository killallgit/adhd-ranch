import { describe, expect, it } from "vitest";
import type { AgentSession } from "../types/agentSession";
import type { Focus } from "../types/focus";
import { animalPen, projectAnimals } from "./animals";

const RANCH_PEN = { id: "/Users/ryan/code/adhd-ranch", name: "adhd-ranch" };

const focus = (overrides: Partial<Focus> & Pick<Focus, "id" | "title">): Focus => ({
  description: "",
  created_at: "",
  tasks: [],
  ...overrides,
});

const session = (overrides: Partial<AgentSession> & Pick<AgentSession, "id">): AgentSession => ({
  name: "adhd-ranch",
  pen: RANCH_PEN,
  activity: "Idle",
  ...overrides,
});

describe("projectAnimals", () => {
  it("puts Focus Animals before Session Animals", () => {
    const animals = projectAnimals(
      [focus({ id: "a", title: "Customer X bug" })],
      [session({ id: "session-1" })],
      0,
    );

    expect(animals.map((animal) => animal.kind)).toEqual(["focus", "agent"]);
  });

  it("lets the clock reach a Focus Animal and no Session Animal", () => {
    const focuses = [
      focus({
        id: "a",
        title: "Customer X bug",
        timer: { duration_secs: 120, started_at: 1_000, status: "Running" },
      }),
    ];
    const sessions = [session({ id: "session-1" })];

    const early = projectAnimals(focuses, sessions, 1_060_000);
    const late = projectAnimals(focuses, sessions, 2_000_000);

    expect(early[0].scale).not.toBe(late[0].scale);
    expect(early[1]).toEqual(late[1]);
  });
});

describe("animalPen", () => {
  it("gives a Session Animal the Pen of its Session", () => {
    const pen = { id: "/Users/ryan/code/other-checkout", name: "other-checkout" };
    const [animal] = projectAnimals([], [session({ id: "session-1", pen })], 0);

    expect(animalPen(animal)).toEqual(pen);
  });

  it("gives a Focus Animal no Pen", () => {
    const [animal] = projectAnimals([focus({ id: "a", title: "Customer X bug" })], [], 0);

    expect(animalPen(animal)).toBeNull();
  });
});
