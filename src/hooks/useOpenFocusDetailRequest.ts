import { useEffect } from "react";
import { subscribeOpenFocusDetail } from "../api/pig";
import type { Focus } from "../types/focus";

export function useOpenFocusDetailRequest(
  focuses: readonly Focus[],
  openFocusDetail: (focusId: string) => void,
) {
  useEffect(() => {
    const unsubscribe = subscribeOpenFocusDetail((focusId) => {
      if (focuses.some((focus) => focus.id === focusId)) {
        openFocusDetail(focusId);
      }
    }).catch(() => () => {});
    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, [focuses, openFocusDetail]);
}
