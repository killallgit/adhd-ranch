import type { TimerPreset } from "./generated/TimerPreset";

export type { FocusTimer } from "./generated/FocusTimer";
export type { TimerPreset } from "./generated/TimerPreset";
export type { TimerStatus } from "./generated/TimerStatus";

export type TimerPresetVariant = Exclude<TimerPreset, { Custom: number }>;

export const PRESET_OPTIONS: Array<{ label: string; value: TimerPreset | null }> = [
  { label: "No timer", value: null },
  { label: "2m", value: "Two" },
  { label: "4m", value: "Four" },
  { label: "8m", value: "Eight" },
  { label: "16m", value: "Sixteen" },
  { label: "32m", value: "ThirtyTwo" },
  { label: "Custom", value: null },
];
