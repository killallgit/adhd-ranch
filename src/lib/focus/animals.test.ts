import { describe, expect, it } from "vitest";
import type { Focus } from "../../types/focus";
import { animalScale, projectFocusAnimals } from "./animals";

const focus = (overrides: Partial<Focus> & Pick<Focus, "id" | "title">): Focus => ({
  description: "",
  created_at: "",
  tasks: [],
  ...overrides,
});

const EXPIRED_TIMER = { duration_secs: 120, started_at: 1_000, status: "Expired" } as const;

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

describe("projectFocusAnimals", () => {
  it("projects a Focus as an Animal named after its title", () => {
    const [animal] = projectFocusAnimals([focus({ id: "a", title: "Customer X bug" })], 0);

    expect(animal).toMatchObject({ kind: "focus", id: "a", name: "Customer X bug" });
  });

  it("rests an Animal when its Focus Timer expired", () => {
    const [animal] = projectFocusAnimals(
      [focus({ id: "a", title: "Customer X bug", timer: EXPIRED_TIMER })],
      0,
    );

    expect(animal.resting).toBe(true);
  });

  it("does not rest an Animal for an expired Task Timer", () => {
    const [animal] = projectFocusAnimals(
      [
        focus({
          id: "a",
          title: "Customer X bug",
          tasks: [{ id: "task-1", text: "Write tests", done: false, timer: EXPIRED_TIMER }],
        }),
      ],
      0,
    );

    expect(animal.resting).toBe(false);
  });

  it("scales an Animal by its Timer progress", () => {
    const [animal] = projectFocusAnimals(
      [
        focus({
          id: "a",
          title: "Customer X bug",
          timer: { duration_secs: 120, started_at: 1_000, status: "Running" },
        }),
      ],
      1_060_000,
    );

    expect(animal.scale).toBe(2);
  });
});
