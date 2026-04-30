import { useEffect, useState } from "react";
import type { FocusReader, Unsubscribe } from "../api/focuses";
import type { Focus } from "../types/focus";

export type FocusesState =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly focuses: readonly Focus[] }
  | { readonly status: "error"; readonly error: Error };

export function useFocuses(reader: FocusReader): FocusesState {
  const [state, setState] = useState<FocusesState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    let unsubscribe: Unsubscribe | null = null;

    const refresh = () => {
      reader
        .list()
        .then((focuses) => {
          if (!cancelled) setState({ status: "ready", focuses });
        })
        .catch((error: unknown) => {
          if (!cancelled)
            setState({
              status: "error",
              error: error instanceof Error ? error : new Error(String(error)),
            });
        });
    };

    refresh();

    if (reader.subscribe) {
      Promise.resolve(reader.subscribe(refresh)).then((un) => {
        if (cancelled) {
          un();
          return;
        }
        unsubscribe = un;
      });
    }

    return () => {
      cancelled = true;
      if (unsubscribe) unsubscribe();
    };
  }, [reader]);

  return state;
}
