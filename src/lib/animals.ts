import type { AgentSession } from "../types/agentSession";
import type { Animal } from "../types/animal";
import type { Focus } from "../types/focus";
import type { Pen } from "../types/generated/Pen";
import { projectFocusAnimals } from "./focus/animals";
import { projectSessionAnimals } from "./session/animals";

// The seam between the ranch's two halves. The clock reaches only the timed one.
export function projectAnimals(
  focuses: readonly Focus[],
  sessions: readonly AgentSession[],
  nowMs: number,
): readonly Animal[] {
  return [...projectFocusAnimals(focuses, nowMs), ...projectSessionAnimals(sessions)];
}

export function animalPen(animal: Animal): Pen | null {
  return animal.kind === "agent" ? animal.session.pen : null;
}
