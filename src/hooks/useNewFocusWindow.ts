import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import { type PresetSelection, isCustomValid, resolvePreset } from "../lib/timerPreset";

export interface NewFocusWindowState {
  readonly title: string;
  readonly description: string;
  readonly timerSelection: PresetSelection;
  readonly customMinutes: number;
  readonly submitting: boolean;
  readonly error: string | null;
  readonly setTitle: (v: string) => void;
  readonly setDescription: (v: string) => void;
  readonly setTimerSelection: (v: PresetSelection) => void;
  readonly setCustomMinutes: (v: number) => void;
  readonly handleSubmit: (e: React.FormEvent<HTMLFormElement>) => Promise<void>;
  readonly handleCancel: () => Promise<void>;
}

export function useNewFocusWindow(focusWriter: FocusWriter): NewFocusWindowState {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [timerSelection, setTimerSelection] = useState<PresetSelection>("none");
  const [customMinutes, setCustomMinutes] = useState(10);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const resetForm = () => {
    setTitle("");
    setDescription("");
    setTimerSelection("none");
    setCustomMinutes(10);
    setError(null);
  };

  const hideWindow = async () => {
    resetForm();
    await getCurrentWindow().hide();
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setTitle("");
        setDescription("");
        setTimerSelection("none");
        setCustomMinutes(10);
        setError(null);
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (title.trim().length === 0) {
      setError("title is required");
      return;
    }
    if (timerSelection === "custom" && !isCustomValid(customMinutes)) {
      setError("custom timer must be at least 1 minute");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const outcome = await focusWriter.createFocus({
        title: title.trim(),
        description: description.trim(),
        timer_preset: resolvePreset(timerSelection, customMinutes),
      });
      if (outcome.ok) {
        await hideWindow();
      } else {
        setError(outcome.message);
      }
    } finally {
      setSubmitting(false);
    }
  };

  const handleCancel = async () => {
    await hideWindow();
  };

  return {
    title,
    description,
    timerSelection,
    customMinutes,
    submitting,
    error,
    setTitle,
    setDescription,
    setTimerSelection,
    setCustomMinutes,
    handleSubmit,
    handleCancel,
  };
}
