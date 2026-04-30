import { useEffect, useState } from "react";
import type { Unsubscribe } from "../api/focuses";
import type { ProposalReader } from "../api/proposals";
import type { Proposal } from "../types/proposal";

export type ProposalsState =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly proposals: readonly Proposal[] }
  | { readonly status: "error"; readonly error: Error };

export function useProposals(reader: ProposalReader): ProposalsState {
  const [state, setState] = useState<ProposalsState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    let unsubscribe: Unsubscribe | null = null;

    const refresh = () => {
      reader
        .list()
        .then((proposals) => {
          if (!cancelled) setState({ status: "ready", proposals });
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
