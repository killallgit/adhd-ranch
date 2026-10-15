import { useCallback, useEffect, useRef, useState } from "react";
import { DEFAULT_TIMER_MINUTES, isBuiltinTimerMinute, minutesToPreset } from "../lib/timerPreset";
import type { FocusTimer, TimerPreset } from "../types/timer";
import { TimerPresetPicker } from "./TimerPresetPicker";

export interface TimerDropdownProps {
  readonly timer?: FocusTimer | null;
  readonly ariaLabel: string;
  readonly savedMinutes: readonly number[];
  readonly onStart: (preset: TimerPreset) => void;
  readonly onAddSaved: (minutes: number) => void;
  readonly onRemoveSaved: (minutes: number) => void;
  readonly onClearSaved: () => void;
  readonly onOpen?: () => void;
  readonly onClear?: () => void;
}

export function TimerDropdown({
  timer,
  ariaLabel,
  savedMinutes,
  onStart,
  onAddSaved,
  onRemoveSaved,
  onClearSaved,
  onOpen,
  onClear,
}: TimerDropdownProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const pointerCommitRef = useRef(false);
  const drafting = useRef(false);
  const [open, setOpen] = useState(false);
  const [selectedMinutes, setSelectedMinutes] = useState(DEFAULT_TIMER_MINUTES);
  const [draftNonce, setDraftNonce] = useState(0);

  const onDraftChange = useCallback((adding: boolean) => {
    drafting.current = adding;
  }, []);

  const start = useCallback(() => {
    const preset = minutesToPreset(selectedMinutes);
    if (preset === null) return;
    onStart(preset);
    setOpen(false);
  }, [onStart, selectedMinutes]);

  const cancelDraft = useCallback(() => {
    setDraftNonce((nonce) => nonce + 1);
    setOpen(false);
  }, []);

  useEffect(() => {
    if (!open) return;
    function handlePointerDown(e: PointerEvent) {
      const target = e.target;
      if (!(target instanceof Node)) return;
      if (rootRef.current?.contains(target)) return;
      if (drafting.current) {
        cancelDraft();
        return;
      }
      pointerCommitRef.current = true;
      start();
      window.setTimeout(() => {
        pointerCommitRef.current = false;
      }, 0);
    }
    window.addEventListener("pointerdown", handlePointerDown, true);
    return () => window.removeEventListener("pointerdown", handlePointerDown, true);
  }, [cancelDraft, open, start]);

  return (
    <div
      ref={rootRef}
      className="timer-dropdown"
      onBlur={(e) => {
        const next = e.relatedTarget;
        if (!open || (next instanceof Node && e.currentTarget.contains(next))) return;
        if (pointerCommitRef.current) return;
        if (drafting.current) {
          cancelDraft();
          return;
        }
        start();
      }}
      onKeyDown={(e) => {
        if (!open || e.key !== "Enter") return;
        e.preventDefault();
        if (drafting.current) return;
        start();
      }}
    >
      <button
        type="button"
        className={`timer-trigger${timer ? " timer-trigger--set" : ""}`}
        aria-label={ariaLabel}
        aria-expanded={open}
        onClick={() => {
          setOpen((value) => {
            const next = !value;
            if (next) onOpen?.();
            return next;
          });
        }}
      >
        {timer ? formatTimer(timer) : <span aria-hidden="true">◷</span>}
      </button>
      {open && (
        <div className="timer-dropdown-menu">
          <TimerPresetPicker
            savedMinutes={savedMinutes}
            selectedMinutes={selectedMinutes}
            allowNone={false}
            draftNonce={draftNonce}
            onDraftChange={onDraftChange}
            onSelect={(minutes) => {
              if (minutes !== null) setSelectedMinutes(minutes);
            }}
            onAdd={onAddSaved}
            onRemove={(minutes) => {
              onRemoveSaved(minutes);
              if (selectedMinutes === minutes) setSelectedMinutes(DEFAULT_TIMER_MINUTES);
            }}
            onClearSaved={() => {
              onClearSaved();
              if (!isBuiltinTimerMinute(selectedMinutes)) {
                setSelectedMinutes(DEFAULT_TIMER_MINUTES);
              }
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
        </div>
      )}
    </div>
  );
}

function formatTimer(timer: FocusTimer): string {
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
