import type { HookFiring } from "../../types/generated/HookFiring";

// A firing is read at a glance while something else is happening on screen, so the
// date is dropped — anything old enough for the date to matter has scrolled away.
export function clockTime(at: string): string {
  const parsed = new Date(at);
  return Number.isNaN(parsed.getTime()) ? at : parsed.toLocaleTimeString([], { hour12: false });
}

// Session ids are UUIDs and every one of them looks the same until the last few
// characters. Showing the head is what lets two sessions be told apart in a list.
export function shortId(id: string | null): string {
  if (!id) return "unreadable";
  return id.length <= 8 ? id : `${id.slice(0, 8)}…`;
}

// Newest first: the question being asked is always "what just happened".
export function newestFirst(firings: readonly HookFiring[]): readonly HookFiring[] {
  return [...firings].reverse();
}
