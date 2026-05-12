import { type PresetSelection, TimerPresetPicker } from "./TimerPresetPicker";

export interface NewFocusWindowProps {
  readonly title: string;
  readonly description: string;
  readonly timerSelection: PresetSelection;
  readonly customMinutes: number;
  readonly submitting: boolean;
  readonly error: string | null;
  readonly onTitleChange: (v: string) => void;
  readonly onDescriptionChange: (v: string) => void;
  readonly onTimerSelectionChange: (v: PresetSelection) => void;
  readonly onCustomMinutesChange: (v: number) => void;
  readonly onSubmit: (e: React.FormEvent<HTMLFormElement>) => void;
  readonly onCancel: () => void;
}

export function NewFocusWindow({
  title,
  description,
  timerSelection,
  customMinutes,
  submitting,
  error,
  onTitleChange,
  onDescriptionChange,
  onTimerSelectionChange,
  onCustomMinutesChange,
  onSubmit,
  onCancel,
}: NewFocusWindowProps) {
  return (
    <form className="new-focus-form new-focus-form--window" onSubmit={onSubmit}>
      <input
        type="text"
        placeholder="Title"
        aria-label="new focus title"
        value={title}
        onChange={(e) => onTitleChange(e.target.value)}
        disabled={submitting}
        // biome-ignore lint/a11y/noAutofocus: first field in a small dedicated window
        autoFocus
      />
      <input
        type="text"
        placeholder="Description"
        aria-label="new focus description"
        value={description}
        onChange={(e) => onDescriptionChange(e.target.value)}
        disabled={submitting}
      />
      <TimerPresetPicker
        selection={timerSelection}
        customMinutes={customMinutes}
        disabled={submitting}
        onSelectionChange={onTimerSelectionChange}
        onCustomMinutesChange={onCustomMinutesChange}
      />
      <div className="new-focus-actions">
        <button type="submit" disabled={submitting}>
          Create
        </button>
        <button type="button" onClick={onCancel} disabled={submitting}>
          Cancel
        </button>
      </div>
      {error && (
        <p data-testid="new-focus-error" role="alert" className="new-focus-error">
          {error}
        </p>
      )}
    </form>
  );
}
