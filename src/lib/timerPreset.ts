import type { TimerPreset } from "../types/timer";

export const BUILTIN_TIMER_MINUTES = [10, 30, 60] as const;

export const DEFAULT_TIMER_MINUTES = 30;

export function isBuiltinTimerMinute(minutes: number): boolean {
  return (BUILTIN_TIMER_MINUTES as readonly number[]).includes(minutes);
}

export function minutesToPreset(minutes: number | null): TimerPreset | null {
  if (minutes === null) return null;
  return { Custom: minutes };
}

export function isCustomValid(minutes: number): boolean {
  return Number.isFinite(minutes) && Number.isInteger(minutes) && minutes >= 1;
}

/** Built-ins and duplicates are already on the picker, so they are not stored again. */
export function rememberTimerMinute(saved: readonly number[], minutes: number): readonly number[] {
  if (!isCustomValid(minutes) || isBuiltinTimerMinute(minutes) || saved.includes(minutes)) {
    return saved;
  }
  return [...saved, minutes];
}

export function forgetTimerMinute(saved: readonly number[], minutes: number): readonly number[] {
  if (!saved.includes(minutes)) return saved;
  return saved.filter((savedMinute) => savedMinute !== minutes);
}
