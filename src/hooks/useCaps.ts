import { useEffect, useState } from "react";
import { type Caps, type CapsReader, DEFAULT_CAPS } from "../api/caps";

export function useCaps(reader: CapsReader): Caps {
  const [caps, setCaps] = useState<Caps>(DEFAULT_CAPS);

  useEffect(() => {
    let cancelled = false;
    reader
      .get()
      .then((next) => {
        if (!cancelled) setCaps(next);
      })
      .catch(() => {
        // fall back to defaults; widget still functions
      });
    return () => {
      cancelled = true;
    };
  }, [reader]);

  return caps;
}
