import { describe, expect, it } from "vitest";
import type { Focus } from "../types/focus";
import { animalScale, projectAnimals } from "./animals";

const focus = (overrides: Partial<Focus> & Pick<Focus, "id" | "title">): Focus => ({
  description: "",
  created_at: "",
  tasks: [],
  ...overrides,
});

describe("animalScale", () => {
  it("returns 1 for a Focus without a Timer", () => {
    expect(animalScale(null, null, 0)).toBe(1);
  });

  it("grows linearly from 1 to 3 over the Timer duration", () => {
    expect(animalScale(1_000, 120, 1_060_000)).toBe(2);
  });

  it("clamps to 3 at the Timer duration", () => {
    expect(animalScale(1_000, 120, 1_120_000)).toBe(3);
  });

  it("stays at 3 past the Timer duration", () => {
    expect(animalScale(1_000, 120, 2_000_000)).toBe(3);
  });
});

describe("projectAnimals", () => {
  it("projects a Focus as a selectable Animal named after its title", () => {
    const [animal] = projectAnimals([focus({ id: "a", title: "Customer X bug" })], [], 0);

    expect(animal).toMatchObject({ kind: "focus", id: "a", name: "Customer X bug" });
  });

  it("gives an Agent Session its own id space and its session name", () => {
    const [animal] = projectAnimals([], [{ id: "session-1", name: "adhd-ranch" }], 0);

    expect(animal).toMatchObject({ kind: "agent", id: "agent:session-1", name: "adhd-ranch" });
  });

  it("puts Focus Animals before agent Animals", () => {
    const animals = projectAnimals(
      [focus({ id: "a", title: "Customer X bug" })],
      [{ id: "session-1", name: "adhd-ranch" }],
      0,
    );

    expect(animals.map((animal) => animal.kind)).toEqual(["focus", "agent"]);
  });

  it("marks an Animal expired when its Focus Timer expired", () => {
    const [animal] = projectAnimals(
      [
        focus({
          id: "a",
          title: "Customer X bug",
          timer: { duration_secs: 120, started_at: 1_000, status: "Expired" },
        }),
      ],
      [],
      0,
    );

    expect(animal?.expired).toBe(true);
  });

  it("does not mark an Animal expired for an expired Task Timer", () => {
    const [animal] = projectAnimals(
      [
        focus({
          id: "a",
          title: "Customer X bug",
          tasks: [
            {
              id: "task-1",
              text: "Write tests",
              done: false,
              timer: { duration_secs: 120, started_at: 1_000, status: "Expired" },
            },
          ],
        }),
      ],
      [],
      0,
    );

    expect(animal?.expired).toBe(false);
  });

  it("scales a Focus Animal by its Timer progress", () => {
    const [animal] = projectAnimals(
      [
        focus({
          id: "a",
          title: "Customer X bug",
          timer: { duration_secs: 120, started_at: 1_000, status: "Running" },
        }),
      ],
      [],
      1_060_000,
    );

    expect(animal?.scale).toBe(2);
  });

  it("never scales an agent Animal", () => {
    const [animal] = projectAnimals([], [{ id: "session-1", name: "adhd-ranch" }], 9_999_999);

    expect(animal?.scale).toBe(1);
  });
});
