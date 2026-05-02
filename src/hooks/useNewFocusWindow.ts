import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import type { FocusWriter } from "../api/focusWriter";

export interface NewFocusWindowState {
  readonly title: string;
  readonly description: string;
  readonly submitting: boolean;
  readonly error: string | null;
  readonly setTitle: (v: string) => void;
  readonly setDescription: (v: string) => void;
  readonly handleSubmit: (e: React.FormEvent<HTMLFormElement>) => Promise<void>;
  readonly handleCancel: () => Promise<void>;
}

export function useNewFocusWindow(focusWriter: FocusWriter): NewFocusWindowState {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const hideWindow = async () => {
    setTitle("");
    setDescription("");
    setError(null);
    await getCurrentWindow().hide();
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setTitle("");
        setDescription("");
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
    setSubmitting(true);
    setError(null);
    try {
      await focusWriter.createFocus({ title: title.trim(), description: description.trim() });
      await hideWindow();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
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
    submitting,
    error,
    setTitle,
    setDescription,
    handleSubmit,
    handleCancel,
  };
}
