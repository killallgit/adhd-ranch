export interface NewFocusWindowProps {
  readonly title: string;
  readonly description: string;
  readonly submitting: boolean;
  readonly error: string | null;
  readonly onTitleChange: (v: string) => void;
  readonly onDescriptionChange: (v: string) => void;
  readonly onSubmit: (e: React.FormEvent<HTMLFormElement>) => void;
  readonly onCancel: () => void;
}

export function NewFocusWindow({
  title,
  description,
  submitting,
  error,
  onTitleChange,
  onDescriptionChange,
  onSubmit,
  onCancel,
}: NewFocusWindowProps) {
  return (
    <form className="new-focus-form" onSubmit={onSubmit}>
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
