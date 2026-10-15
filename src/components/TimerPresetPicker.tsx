import { useEffect, useRef, useState } from "react";
import { BUILTIN_TIMER_MINUTES, isBuiltinTimerMinute, isCustomValid } from "../lib/timerPreset";

export interface TimerPresetPickerProps {
  readonly savedMinutes: readonly number[];
  readonly selectedMinutes: number | null;
  readonly disabled?: boolean;
  readonly allowNone?: boolean;
  readonly draftNonce?: number;
  readonly onSelect: (minutes: number | null) => void;
  readonly onAdd: (minutes: number) => void;
  readonly onRemove: (minutes: number) => void;
  readonly onClearSaved: () => void;
  readonly onDraftChange?: (adding: boolean) => void;
}

export function TimerPresetPicker({
  savedMinutes,
  selectedMinutes,
  disabled,
  allowNone = true,
  draftNonce = 0,
  onSelect,
  onAdd,
  onRemove,
  onClearSaved,
  onDraftChange,
}: TimerPresetPickerProps) {
  const [adding, setAdding] = useState(false);
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const seenNonce = useRef(draftNonce);

  useEffect(() => {
    onDraftChange?.(adding);
  }, [adding, onDraftChange]);

  useEffect(() => {
    if (seenNonce.current === draftNonce) return;
    seenNonce.current = draftNonce;
    setAdding(false);
    setDraft("");
    setError(null);
  }, [draftNonce]);

  function select(minutes: number | null) {
    setAdding(false);
    setDraft("");
    setError(null);
    onSelect(minutes);
  }

  function commitDraft() {
    const minutes = Number(draft);
    if (!isCustomValid(minutes)) {
      setError("custom timer must be at least 1 minute");
      return;
    }
    setError(null);
    setAdding(false);
    setDraft("");
    onSelect(minutes);
    if (!isBuiltinTimerMinute(minutes) && !savedMinutes.includes(minutes)) {
      onAdd(minutes);
    }
  }

  const extras = savedMinutes.filter((minutes) => !isBuiltinTimerMinute(minutes));

  return (
    <div className="timer-preset-picker">
      <fieldset className="timer-chips" aria-label="timer preset">
        {allowNone && (
          <button
            type="button"
            className="timer-chip"
            aria-pressed={selectedMinutes === null}
            disabled={disabled}
            onClick={() => select(null)}
          >
            None
          </button>
        )}
        {BUILTIN_TIMER_MINUTES.map((minutes) => (
          <TimerChip
            key={minutes}
            minutes={minutes}
            pressed={selectedMinutes === minutes}
            disabled={disabled}
            onSelect={() => select(minutes)}
          />
        ))}
        {extras.map((minutes) => (
          <span key={minutes} className="timer-saved">
            <TimerChip
              minutes={minutes}
              pressed={selectedMinutes === minutes}
              disabled={disabled}
              onSelect={() => select(minutes)}
            />
            <button
              type="button"
              className="timer-chip-remove"
              aria-label={`remove ${minutes} minute timer`}
              disabled={disabled}
              onClick={() => onRemove(minutes)}
            >
              ×
            </button>
          </span>
        ))}
        {adding ? (
          <span className="timer-chip-draft">
            <input
              type="number"
              aria-label="custom timer minutes"
              data-testid="custom-timer-input"
              min={1}
              value={draft}
              disabled={disabled}
              // biome-ignore lint/a11y/noAutofocus: the field exists only after the user asks to add a duration
              autoFocus
              onChange={(e) => {
                setDraft(e.target.value);
                if (error) setError(null);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  e.stopPropagation();
                  commitDraft();
                } else if (e.key === "Escape") {
                  e.preventDefault();
                  e.stopPropagation();
                  setAdding(false);
                  setDraft("");
                  setError(null);
                }
              }}
            />
            <button type="button" disabled={disabled} onClick={commitDraft}>
              Add
            </button>
          </span>
        ) : (
          <button
            type="button"
            className="timer-chip"
            aria-label="add timer"
            disabled={disabled}
            onClick={() => {
              setAdding(true);
              setError(null);
            }}
          >
            +
          </button>
        )}
      </fieldset>
      {extras.length > 0 && (
        <button
          type="button"
          className="timer-saved-clear"
          disabled={disabled}
          onClick={onClearSaved}
        >
          Clear saved
        </button>
      )}
      {error && (
        <p className="timer-dropdown-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}

function TimerChip({
  minutes,
  pressed,
  disabled,
  onSelect,
}: {
  readonly minutes: number;
  readonly pressed: boolean;
  readonly disabled?: boolean;
  readonly onSelect: () => void;
}) {
  return (
    <button
      type="button"
      className="timer-chip"
      aria-label={`${minutes} minutes`}
      aria-pressed={pressed}
      disabled={disabled}
      onClick={onSelect}
    >
      {minutes}m
    </button>
  );
}
