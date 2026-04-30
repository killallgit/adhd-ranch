import { useEffect, useState } from "react";

export type Unsubscribe = () => void;

export interface PolledReader<T> {
  read(): Promise<T>;
  subscribe?(onChange: () => void): Unsubscribe | Promise<Unsubscribe>;
}

export type PolledState<T> =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly value: T }
  | { readonly status: "error"; readonly error: Error };

export function usePolledReader<T>(reader: PolledReader<T>): PolledState<T> {
  const [state, setState] = useState<PolledState<T>>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    let unsubscribe: Unsubscribe | null = null;

    const refresh = () => {
      reader
        .read()
        .then((value) => {
          if (!cancelled) setState({ status: "ready", value });
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
