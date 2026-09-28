import { useCallback, useEffect, useState } from "react";
import { subscribeOpenFocusDetail } from "../api/pig";
import type { Focus } from "../types/focus";

export interface FocusSelection {
  readonly selected: Focus | null;
  readonly request: (renderId: string) => void;
  readonly close: () => void;
}

export function useFocusSelection(focuses: readonly Focus[]): FocusSelection {
  const [requestedId, setRequestedId] = useState<string | null>(null);
  const request = useCallback((renderId: string) => setRequestedId(renderId), []);
  const close = useCallback(() => setRequestedId(null), []);

  useEffect(() => {
    const unsubscribe = subscribeOpenFocusDetail(request).catch(() => () => {});
    return () => {
      unsubscribe.then((unlisten) => unlisten());
    };
  }, [request]);

  const selected = focuses.find((focus) => focus.id === requestedId) ?? null;

  useEffect(() => {
    if (requestedId !== null && selected === null) setRequestedId(null);
  }, [requestedId, selected]);

  return { selected, request, close };
}
