import type { Caps } from "../api/caps";
import type { Focus } from "../types/focus";

export interface CapState {
  readonly focusesOver: boolean;
  readonly focusCount: number;
  readonly overTaskFocusIds: readonly string[];
  readonly anyOver: boolean;
}

export function computeCapState(focuses: readonly Focus[], caps: Caps): CapState {
  const focusesOver = focuses.length > caps.max_focuses;
  const overTaskFocusIds = focuses
    .filter((f) => f.tasks.length > caps.max_tasks_per_focus)
    .map((f) => f.id);
  const anyOver = focusesOver || overTaskFocusIds.length > 0;
  return { focusesOver, focusCount: focuses.length, overTaskFocusIds, anyOver };
}
