export type TimerPresetVariant = "Two" | "Four" | "Eight" | "Sixteen" | "ThirtyTwo";

export type TimerPreset = TimerPresetVariant | { Custom: number }; // minutes

export type TimerStatus = "Running" | "Expired";

export interface FocusTimer {
  readonly duration_secs: number;
  readonly started_at: number;
  readonly status: TimerStatus;
}

export const PRESET_OPTIONS: Array<{ label: string; value: TimerPreset | null }> = [
  { label: "No timer", value: null },
  { label: "2m", value: "Two" },
  { label: "4m", value: "Four" },
  { label: "8m", value: "Eight" },
  { label: "16m", value: "Sixteen" },
  { label: "32m", value: "ThirtyTwo" },
  { label: "Custom", value: null }, // placeholder — custom uses number input
];
