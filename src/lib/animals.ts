import type { AgentSession } from "../types/agentSession";
import type { Animal } from "../types/animal";
import type { Focus } from "../types/focus";

// Agent Session ids and Focus ids come from different sources, so agent Animals
// get their own id space on the overlay.
function agentAnimalId(session: AgentSession): string {
  return `agent:${session.id}`;
}

export function animalScale(
  startedAt: number | null,
  durationSecs: number | null,
  nowMs: number,
): number {
  if (startedAt === null || durationSecs === null) return 1;
  if (durationSecs <= 0) return 3;
  const elapsedSecs = Math.max(0, nowMs / 1000 - startedAt);
  const progress = Math.min(1, elapsedSecs / durationSecs);
  return 1 + progress * 2;
}

export function projectAnimals(
  focuses: readonly Focus[],
  sessions: readonly AgentSession[],
  nowMs: number,
): readonly Animal[] {
  return [
    ...focuses.map(
      (focus): Animal => ({
        kind: "focus",
        id: focus.id,
        name: focus.title,
        expired: focus.timer?.status === "Expired",
        scale: animalScale(
          focus.timer?.started_at ?? null,
          focus.timer?.duration_secs ?? null,
          nowMs,
        ),
        focus,
      }),
    ),
    ...sessions.map(
      (session): Animal => ({
        kind: "agent",
        id: agentAnimalId(session),
        name: session.name,
        expired: false,
        scale: 1,
        session,
      }),
    ),
  ];
}
