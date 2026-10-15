import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import { useSavedTimerMinutes } from "../hooks/useSavedTimerMinutes";
import { minutesToPreset } from "../lib/timerPreset";

export function useNewFocusWindow(focusWriter: FocusWriter) {
  const savedTimers = useSavedTimerMinutes();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [selectedMinutes, setSelectedMinutes] = useState<number | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const resetForm = () => {
    setTitle("");
    setDescription("");
    setSelectedMinutes(null);
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
        setSelectedMinutes(null);
        setError(null);
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    let stop: (() => void) | undefined;
    let cancelled = false;
    void getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused) savedTimers.reload();
      })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else stop = unlisten;
      });
    return () => {
      cancelled = true;
      stop?.();
    };
  }, [savedTimers.reload]);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (title.trim().length === 0) {
      setError("title is required");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const outcome = await focusWriter.createFocus({
        title: title.trim(),
        description: description.trim(),
        timer_preset: minutesToPreset(selectedMinutes),
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
    selectedMinutes,
    savedTimers,
    submitting,
    error,
    setTitle,
    setDescription,
    setSelectedMinutes,
    handleSubmit,
    handleCancel,
  };
}
