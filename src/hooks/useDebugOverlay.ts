import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

function getTopOffset(): number {
  if (typeof navigator === "undefined") return 0;
  return /mac/i.test(navigator.platform) ? 28 : 0;
}

const TOP_OFFSET = getTopOffset();

export function useDebugOverlay() {
  const [visible, setVisible] = useState(import.meta.env.DEV);

  useEffect(() => {
    const unlisten = listen<boolean>("debug-overlay-toggle", (e) => {
      setVisible(e.payload);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return { visible, topOffset: TOP_OFFSET };
}
