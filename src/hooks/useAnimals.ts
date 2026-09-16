import { projectAnimals } from "../lib/animals";
import type { AgentSession } from "../types/agentSession";
import type { Animal } from "../types/animal";
import type { Focus } from "../types/focus";

// An Animal's scale grows with the clock, so the projection is resampled on every
// render; the movement loop is what keeps those renders coming. Reading the clock
// here keeps it out of the components.
export function useAnimals(
  focuses: readonly Focus[],
  sessions: readonly AgentSession[],
): readonly Animal[] {
  return projectAnimals(focuses, sessions, Date.now());
}
