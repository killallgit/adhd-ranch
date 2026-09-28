import type { AnimalSize } from "../types/generated/AnimalSize";

export function sampleAnimalSize(size: AnimalSize, nowMs: number): number {
  switch (size.kind) {
    case "fixed":
      return positiveFinite(size.px);
    case "linear": {
      const fromPx = positiveFinite(size.fromPx);
      const toPx = positiveFinite(size.toPx);
      if (size.durationMs <= 0 || !Number.isFinite(size.durationMs)) {
        throw new RangeError("linear animal size duration must be finite and positive");
      }
      const progress = Math.min(1, Math.max(0, (nowMs - size.startedAtMs) / size.durationMs));
      return fromPx + (toPx - fromPx) * progress;
    }
  }
}

function positiveFinite(value: number): number {
  if (!Number.isFinite(value) || value <= 0) {
    throw new RangeError("animal size must be finite and positive");
  }
  return value;
}
