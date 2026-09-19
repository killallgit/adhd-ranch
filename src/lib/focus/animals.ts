import type { FocusAnimal } from "../../types/animal";
import type { Focus } from "../../types/focus";

// A Focus animal is the timed half of the ranch: it grows while its timer runs and
// rests when the timer expires. Nothing here knows about Claude sessions.
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

export function projectFocusAnimals(
  focuses: readonly Focus[],
  nowMs: number,
): readonly FocusAnimal[] {
  return focuses.map((focus) => ({
    kind: "focus",
    id: focus.id,
    name: focus.title,
    resting: focus.timer?.status === "Expired",
    scale: animalScale(focus.timer?.started_at ?? null, focus.timer?.duration_secs ?? null, nowMs),
    focus,
  }));
}
