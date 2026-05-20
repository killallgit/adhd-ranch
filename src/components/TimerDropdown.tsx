import { useCallback, useEffect, useRef, useState } from "react";
import { type PresetSelection, isCustomValid, resolvePreset } from "../lib/timerPreset";
import type { FocusTimer, TimerPreset } from "../types/timer";
import { TimerPresetPicker } from "./TimerPresetPicker";

export interface TimerDropdownProps {
  readonly timer?: FocusTimer | null;
  readonly ariaLabel: string;
  readonly onStart: (preset: TimerPreset) => void;
  readonly onClear?: () => void;
}

export function TimerDropdown({ timer, ariaLabel, onStart, onClear }: TimerDropdownProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [selection, setSelection] = useState<PresetSelection>("Eight");
  const [customMinutes, setCustomMinutes] = useState(10);
  const [error, setError] = useState<string | null>(null);

  const start = useCallback(() => {
    if (selection === "custom" && !isCustomValid(customMinutes)) {
      setError("custom timer must be at least 1 minute");
      return;
    }
    const preset = resolvePreset(selection, customMinutes);
    if (preset === null) return;
    setError(null);
    onStart(preset);
    setOpen(false);
  }, [customMinutes, onStart, selection]);

  useEffect(() => {
    if (!open) return;
    function handlePointerDown(e: PointerEvent) {
      const target = e.target;
      if (!(target instanceof Node)) return;
      if (rootRef.current?.contains(target)) return;
      start();
    }
    window.addEventListener("pointerdown", handlePointerDown, true);
    return () => window.removeEventListener("pointerdown", handlePointerDown, true);
  }, [open, start]);

  return (
    <div
      ref={rootRef}
      className="timer-dropdown"
      onBlur={(e) => {
        const next = e.relatedTarget;
        if (!open || (next instanceof Node && e.currentTarget.contains(next))) return;
        start();
      }}
      onKeyDown={(e) => {
        if (!open || e.key !== "Enter") return;
        e.preventDefault();
        start();
      }}
    >
      <button
        type="button"
        className={`timer-trigger${timer ? " timer-trigger--set" : ""}`}
        aria-label={ariaLabel}
        aria-expanded={open}
        onClick={() => {
          setOpen((value) => !value);
          setError(null);
        }}
      >
        {timer ? formatTimer(timer) : <span aria-hidden="true">◷</span>}
      </button>
      {open && (
        <div className="timer-dropdown-menu">
          <TimerPresetPicker
            selection={selection}
            customMinutes={customMinutes}
            allowNone={false}
            onSelectionChange={(value) => {
              setSelection(value);
              if (error) setError(null);
            }}
            onCustomMinutesChange={(value) => {
              setCustomMinutes(value);
              if (error) setError(null);
            }}
          />
          <div className="timer-dropdown-actions">
            <button type="button" className="timer-dropdown-start" onClick={start}>
              {timer ? "Restart" : "Start"}
            </button>
            {timer && onClear && (
              <button
                type="button"
                className="timer-dropdown-clear"
                onClick={() => {
                  onClear();
                  setOpen(false);
                }}
              >
                Clear
              </button>
            )}
          </div>
          {error && (
            <p className="timer-dropdown-error" role="alert">
              {error}
            </p>
          )}
        </div>
      )}
    </div>
  );
}

export function formatTimer(timer: FocusTimer): string {
  if (timer.status === "Expired") return "Expired";
  const elapsedSecs = Math.max(0, Math.floor(Date.now() / 1000) - timer.started_at);
  const remainingSecs = timer.duration_secs - elapsedSecs;
  if (remainingSecs <= 0) return "Expired";
  const minutes = Math.floor(remainingSecs / 60)
    .toString()
    .padStart(2, "0");
  const seconds = (remainingSecs % 60).toString().padStart(2, "0");
  return `${minutes}:${seconds}`;
}
