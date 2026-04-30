import { useState } from "react";
import type { Focus } from "../types/focus";
import { TaskRow } from "./TaskRow";

export interface FocusCardProps {
  readonly focus: Focus;
  readonly onClearTask: (focusId: string, index: number) => void;
  readonly onDeleteFocus: (focusId: string) => void;
  readonly busy: boolean;
}

export function FocusCard({ focus, onClearTask, onDeleteFocus, busy }: FocusCardProps) {
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  return (
    <section data-testid="focus-card" className="focus-card" aria-label={focus.title}>
      <header className="focus-header">
        <h2 className="focus-title">{focus.title}</h2>
        <button
          type="button"
          className="focus-menu"
          aria-label={`menu ${focus.title}`}
          aria-expanded={confirmingDelete}
          disabled={busy}
          onClick={() => setConfirmingDelete((prev) => !prev)}
        >
          …
        </button>
      </header>
      {confirmingDelete && (
        <div data-testid="focus-delete-confirm" className="focus-delete-confirm" role="alert">
          <span>Delete "{focus.title}"?</span>
          <button
            type="button"
            className="focus-delete-confirm-yes"
            disabled={busy}
            onClick={() => {
              setConfirmingDelete(false);
              onDeleteFocus(focus.id);
            }}
          >
            Delete
          </button>
          <button
            type="button"
            className="focus-delete-confirm-no"
            disabled={busy}
            onClick={() => setConfirmingDelete(false)}
          >
            Cancel
          </button>
        </div>
      )}
      <ul className="task-list">
        {focus.tasks.map((task, index) => (
          <TaskRow
            key={task.id}
            task={task}
            disabled={busy}
            onClear={() => onClearTask(focus.id, index)}
          />
        ))}
      </ul>
    </section>
  );
}
