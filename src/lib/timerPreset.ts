import type { TimerPreset } from "../types/timer";

export type PresetSelection =
  | "none"
  | "Two"
  | "Four"
  | "Eight"
  | "Sixteen"
  | "ThirtyTwo"
  | "custom";

export function resolvePreset(
  selection: PresetSelection,
  customMinutes: number,
): TimerPreset | null {
  switch (selection) {
    case "Two":
      return "Two";
    case "Four":
      return "Four";
    case "Eight":
      return "Eight";
    case "Sixteen":
      return "Sixteen";
    case "ThirtyTwo":
      return "ThirtyTwo";
    case "custom":
      return { Custom: customMinutes };
    default:
      return null;
  }
}

export function isCustomValid(customMinutes: number): boolean {
  return Number.isFinite(customMinutes) && Number.isInteger(customMinutes) && customMinutes >= 1;
}
