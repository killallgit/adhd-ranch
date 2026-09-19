import { describe, expect, it } from "vitest";
import type { AgentSession } from "../../types/agentSession";
import { projectSessionAnimals } from "./animals";

const RANCH_PEN = { id: "/Users/ryan/code/adhd-ranch", name: "adhd-ranch" };

const session = (overrides: Partial<AgentSession> & Pick<AgentSession, "id">): AgentSession => ({
  name: "adhd-ranch",
  pen: RANCH_PEN,
  activity: "Idle",
  ...overrides,
});

describe("projectSessionAnimals", () => {
  it("gives a Session its own id space and its session name", () => {
    const [animal] = projectSessionAnimals([session({ id: "session-1" })]);

    expect(animal).toMatchObject({ kind: "agent", id: "agent:session-1", name: "adhd-ranch" });
  });

  it("keeps a working Session's Animal on its feet", () => {
    const [animal] = projectSessionAnimals([session({ id: "session-1", activity: "Working" })]);

    expect(animal.resting).toBe(false);
  });

  it("rests a Session's Animal between turns", () => {
    const [animal] = projectSessionAnimals([session({ id: "session-1", activity: "Idle" })]);

    expect(animal.resting).toBe(true);
  });

  it("never scales a Session's Animal", () => {
    const [animal] = projectSessionAnimals([session({ id: "session-1", activity: "Working" })]);

    expect(animal.scale).toBe(1);
  });

  it("carries the Session's Pen onto its Animal", () => {
    const pen = { id: "/Users/ryan/code/other-checkout", name: "other-checkout" };

    const [animal] = projectSessionAnimals([session({ id: "session-1", pen })]);

    expect(animal.session.pen).toEqual(pen);
  });
});
