import type { Focus } from "../types/focus";
import { FocusCard } from "./FocusCard";

export interface FocusListProps {
  readonly focuses: readonly Focus[];
  readonly onClearTask: (focusId: string, index: number) => void;
  readonly onDeleteFocus: (focusId: string) => void;
  readonly busyFocusId: string | null;
}

export function FocusList({ focuses, onClearTask, onDeleteFocus, busyFocusId }: FocusListProps) {
  if (focuses.length === 0) {
    return (
      <div data-testid="focus-list-empty" className="focus-list-empty">
        No focuses yet — create one below.
      </div>
    );
  }
  return (
    <div data-testid="focus-list" className="focus-list">
      {focuses.map((focus) => (
        <FocusCard
          key={focus.id}
          focus={focus}
          busy={busyFocusId === focus.id}
          onClearTask={onClearTask}
          onDeleteFocus={onDeleteFocus}
        />
      ))}
    </div>
  );
}
