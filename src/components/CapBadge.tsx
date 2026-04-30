import type { CapState } from "../lib/capState";

export interface CapBadgeProps {
  readonly capState: CapState;
  readonly maxFocuses: number;
}

export function CapBadge({ capState, maxFocuses }: CapBadgeProps) {
  if (!capState.anyOver) return null;

  return (
    <output data-testid="cap-badge" className="cap-badge" aria-live="polite">
      {capState.focusesOver && (
        <span data-testid="cap-badge-focuses">
          {capState.focusCount} focuses (max {maxFocuses}) — trim one
        </span>
      )}
      {capState.overTaskFocusIds.length > 0 && (
        <span data-testid="cap-badge-tasks">
          tasks over: {capState.overTaskFocusIds.join(", ")}
        </span>
      )}
    </output>
  );
}
