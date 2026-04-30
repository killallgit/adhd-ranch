import type { Task } from "../types/focus";

export interface TaskRowProps {
  readonly task: Task;
  readonly onClear: () => void;
  readonly disabled?: boolean;
}

export function TaskRow({ task, onClear, disabled }: TaskRowProps) {
  return (
    <li data-testid="task-row" className="task-row">
      <span className="task-text">{task.text}</span>
      <button
        type="button"
        className="task-clear"
        aria-label={`clear task ${task.text}`}
        disabled={disabled}
        onClick={onClear}
      >
        ✗
      </button>
    </li>
  );
}
