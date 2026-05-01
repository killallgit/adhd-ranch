import type { Caps, CapsReader } from "../api/caps";
import type { FocusReader } from "../api/focuses";
import type { ProposalReader } from "../api/proposals";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";
import { useCaps } from "./useCaps";
import { useFocuses } from "./useFocuses";
import { useProposals } from "./useProposals";

export type AppStatus =
  | { readonly status: "loading" }
  | { readonly status: "error"; readonly error: Error }
  | {
      readonly status: "ready";
      readonly focuses: readonly Focus[];
      readonly proposals: readonly Proposal[];
    };

export type AppState = AppStatus & { readonly caps: Caps };

export interface AppStateDeps {
  readonly focusReader: FocusReader;
  readonly proposalReader: ProposalReader;
  readonly capsReader: CapsReader;
}

export function useAppState({ focusReader, proposalReader, capsReader }: AppStateDeps): AppState {
  const focuses = useFocuses(focusReader);
  const proposals = useProposals(proposalReader);
  const caps = useCaps(capsReader);

  if (focuses.status === "error") return { status: "error", error: focuses.error, caps };
  if (proposals.status === "error") return { status: "error", error: proposals.error, caps };
  if (focuses.status === "loading" || proposals.status === "loading") {
    return { status: "loading", caps };
  }
  return {
    status: "ready",
    focuses: focuses.focuses,
    proposals: proposals.proposals,
    caps,
  };
}
