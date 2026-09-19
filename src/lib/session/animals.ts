import type { AgentSession } from "../../types/agentSession";
import type { SessionAnimal } from "../../types/animal";

// A Session animal is the untimed half of the ranch. It takes no clock: a session is
// working from the moment a prompt is submitted until its turn stops, and the hooks
// say which. There is no duration, no expiry and nothing to count down.

// Session ids and Focus ids come from different sources, so session animals get their
// own id space on the overlay.
function sessionAnimalId(session: AgentSession): string {
  return `agent:${session.id}`;
}

export function projectSessionAnimals(sessions: readonly AgentSession[]): readonly SessionAnimal[] {
  return sessions.map((session) => ({
    kind: "agent",
    id: sessionAnimalId(session),
    name: session.name,
    scale: 1,
    resting: session.activity === "Idle",
    session,
  }));
}
