import type { Task } from "../types/focus";

export interface TaskRowProps {
  readonly task: Task;
}

export function TaskRow({ task }: TaskRowProps) {
  return (
    <li data-testid="task-row" className="task-row">
      <span className="task-text">{task.text}</span>
      <button type="button" className="task-clear" aria-label={`clear task ${task.text}`} disabled>
        ✗
      </button>
    </li>
  );
}
