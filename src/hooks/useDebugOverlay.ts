import { useEffect, useState } from "react";
import { subscribeDebugOverlay } from "../api/debugOverlay";

function getTopOffset(): number {
  if (typeof navigator === "undefined") return 0;
  return /mac/i.test(navigator.platform) ? 28 : 0;
}

const TOP_OFFSET = getTopOffset();

export interface DebugOverlayState {
  visible: boolean;
  topOffset: number;
}

export function useDebugOverlay(): DebugOverlayState {
  const [visible, setVisible] = useState(import.meta.env.DEV);

  useEffect(() => {
    const unsub = subscribeDebugOverlay(setVisible);
    return () => {
      unsub.then((f) => f());
    };
  }, []);

  return { visible, topOffset: TOP_OFFSET };
}
