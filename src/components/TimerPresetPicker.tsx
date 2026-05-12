import type { PresetSelection } from "../lib/timerPreset";

export interface TimerPresetPickerProps {
  readonly selection: PresetSelection;
  readonly customMinutes: number;
  readonly disabled?: boolean;
  readonly allowNone?: boolean;
  readonly onSelectionChange: (v: PresetSelection) => void;
  readonly onCustomMinutesChange: (v: number) => void;
}

export function TimerPresetPicker({
  selection,
  customMinutes,
  disabled,
  allowNone = true,
  onSelectionChange,
  onCustomMinutesChange,
}: TimerPresetPickerProps) {
  return (
    <>
      <select
        aria-label="timer preset"
        data-testid="timer-preset-select"
        value={selection}
        onChange={(e) => onSelectionChange(e.target.value as PresetSelection)}
        disabled={disabled}
      >
        {allowNone && <option value="none">No timer</option>}
        <option value="Two">2m</option>
        <option value="Four">4m</option>
        <option value="Eight">8m</option>
        <option value="Sixteen">16m</option>
        <option value="ThirtyTwo">32m</option>
        <option value="custom">Custom</option>
      </select>
      {selection === "custom" && (
        <input
          type="number"
          aria-label="custom timer minutes"
          data-testid="custom-timer-input"
          min={1}
          value={customMinutes}
          onChange={(e) => onCustomMinutesChange(Number(e.target.value))}
          disabled={disabled}
        />
      )}
    </>
  );
}
