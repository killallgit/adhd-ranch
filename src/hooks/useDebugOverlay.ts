import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

const TOP_OFFSET = /mac/i.test(navigator.platform) ? 28 : 0;

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
