import { useState } from "react";

export interface NewFocusFormProps {
  readonly onCreate: (input: { title: string; description: string }) => Promise<void>;
}

export function NewFocusForm({ onCreate }: NewFocusFormProps) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
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

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (title.trim().length === 0) {
      setError("title is required");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await onCreate({ title: title.trim(), description: description.trim() });
      setTitle("");
      setDescription("");
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
