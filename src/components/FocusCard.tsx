import type { Focus } from "../types/focus";
import { TaskRow } from "./TaskRow";

export interface FocusCardProps {
  readonly focus: Focus;
}

export function FocusCard({ focus }: FocusCardProps) {
  return (
    <section data-testid="focus-card" className="focus-card" aria-label={focus.title}>
      <header className="focus-header">
        <h2 className="focus-title">{focus.title}</h2>
        <button type="button" className="focus-menu" aria-label={`menu ${focus.title}`} disabled>
          …
        </button>
      </header>
      <ul className="task-list">
        {focus.tasks.map((task) => (
          <TaskRow key={task.id} task={task} />
        ))}
      </ul>
    </section>
  );
}
