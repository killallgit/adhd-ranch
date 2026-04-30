import { useMemo } from "react";
import type { ProposalReader } from "../api/proposals";
import type { Proposal } from "../types/proposal";
import { type PolledReader, usePolledReader } from "./usePolledReader";

export type ProposalsState =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly proposals: readonly Proposal[] }
  | { readonly status: "error"; readonly error: Error };

export function useProposals(reader: ProposalReader): ProposalsState {
  const polled: PolledReader<readonly Proposal[]> = useMemo(
    () => ({
      read: () => reader.list(),
      subscribe: reader.subscribe?.bind(reader),
    }),
    [reader],
  );
  const state = usePolledReader(polled);
  if (state.status === "ready") return { status: "ready", proposals: state.value };
  return state;
}
