import type { Focus } from "../types/focus";
import { FocusCard } from "./FocusCard";

export interface FocusListProps {
  readonly focuses: readonly Focus[];
}

export function FocusList({ focuses }: FocusListProps) {
  if (focuses.length === 0) {
    return (
      <div data-testid="focus-list-empty" className="focus-list-empty">
        + New Focus
      </div>
    );
  }
  return (
    <div data-testid="focus-list" className="focus-list">
      {focuses.map((focus) => (
        <FocusCard key={focus.id} focus={focus} />
      ))}
    </div>
  );
}
