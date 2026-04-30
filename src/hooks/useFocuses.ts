import { useMemo } from "react";
import type { FocusReader } from "../api/focuses";
import type { Focus } from "../types/focus";
import { type PolledReader, usePolledReader } from "./usePolledReader";

export type FocusesState =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly focuses: readonly Focus[] }
  | { readonly status: "error"; readonly error: Error };

export function useFocuses(reader: FocusReader): FocusesState {
  const polled: PolledReader<readonly Focus[]> = useMemo(
    () => ({
      read: () => reader.list(),
      subscribe: reader.subscribe?.bind(reader),
    }),
    [reader],
  );
  const state = usePolledReader(polled);
  if (state.status === "ready") return { status: "ready", focuses: state.value };
  return state;
}
