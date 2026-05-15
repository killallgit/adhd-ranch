export function ranchAnimalScale(
  startedAt: number | null,
  durationSecs: number | null,
  nowMs = Date.now(),
): number {
  if (startedAt === null || durationSecs === null) return 1;
  if (durationSecs <= 0) return 3;
  const elapsedSecs = Math.max(0, nowMs / 1000 - startedAt);
  const progress = Math.min(1, elapsedSecs / durationSecs);
  return 1 + progress * 2;
}

export const useRanchAnimalScale = ranchAnimalScale;
