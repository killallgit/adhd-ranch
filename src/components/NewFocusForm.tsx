import { useState } from "react";

import type { TimerPreset } from "../types/timer";

type SelectValue = "none" | "Two" | "Four" | "Eight" | "Sixteen" | "ThirtyTwo" | "custom";

export interface NewFocusFormInput {
  readonly title: string;
  readonly description: string;
  readonly timer_preset: TimerPreset | null;
}

export interface NewFocusFormProps {
  readonly onCreate: (input: NewFocusFormInput) => Promise<void>;
}

export function NewFocusForm({ onCreate }: NewFocusFormProps) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [timerSelect, setTimerSelect] = useState<SelectValue>("none");
  const [customMinutes, setCustomMinutes] = useState(10);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!open) {
    return (
      <button
        type="button"
        className="new-focus-toggle"
        data-testid="new-focus-toggle"
        onClick={() => setOpen(true)}
      >
        + New Focus
      </button>
    );
  }

  function resolvePreset(): TimerPreset | null {
    switch (timerSelect) {
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

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (title.trim().length === 0) {
      setError("title is required");
      return;
    }
    if (timerSelect === "custom" && (customMinutes < 1 || !Number.isFinite(customMinutes))) {
      setError("custom timer must be at least 1 minute");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await onCreate({
        title: title.trim(),
        description: description.trim(),
        timer_preset: resolvePreset(),
      });
      setTitle("");
      setDescription("");
      setTimerSelect("none");
      setCustomMinutes(10);
      setOpen(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form data-testid="new-focus-form" className="new-focus-form" onSubmit={handleSubmit}>
      <input
        type="text"
        placeholder="Title"
        aria-label="new focus title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        disabled={submitting}
      />
      <input
        type="text"
        placeholder="Description"
        aria-label="new focus description"
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        disabled={submitting}
      />
      <select
        aria-label="timer preset"
        data-testid="timer-preset-select"
        value={timerSelect}
        onChange={(e) => setTimerSelect(e.target.value as SelectValue)}
        disabled={submitting}
      >
        <option value="none">No timer</option>
        <option value="Two">2m</option>
        <option value="Four">4m</option>
        <option value="Eight">8m</option>
        <option value="Sixteen">16m</option>
        <option value="ThirtyTwo">32m</option>
        <option value="custom">Custom</option>
      </select>
      {timerSelect === "custom" && (
        <input
          type="number"
          aria-label="custom timer minutes"
          data-testid="custom-timer-input"
          min={1}
          value={customMinutes}
          onChange={(e) => setCustomMinutes(Number(e.target.value))}
          disabled={submitting}
        />
      )}
      <div className="new-focus-actions">
        <button type="submit" disabled={submitting}>
          Create
        </button>
        <button
          type="button"
          onClick={() => {
            setOpen(false);
            setError(null);
          }}
          disabled={submitting}
        >
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
