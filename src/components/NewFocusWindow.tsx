import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";

export function NewFocusWindow() {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") void hideWindow();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const hideWindow = async () => {
    setTitle("");
    setDescription("");
    setError(null);
    await getCurrentWindow().hide();
  };

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (title.trim().length === 0) {
      setError("title is required");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await invoke("create_focus", {
        title: title.trim(),
        description: description.trim() || null,
      });
      await hideWindow();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form className="new-focus-form" onSubmit={handleSubmit}>
      <input
        type="text"
        placeholder="Title"
        aria-label="new focus title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        disabled={submitting}
        // biome-ignore lint/a11y/noAutofocus: first field in a small dedicated window
        autoFocus
      />
      <input
        type="text"
        placeholder="Description"
        aria-label="new focus description"
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        disabled={submitting}
      />
      <div className="new-focus-actions">
        <button type="submit" disabled={submitting}>
          Create
        </button>
        <button type="button" onClick={() => void hideWindow()} disabled={submitting}>
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
