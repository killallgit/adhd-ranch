import type { Caps, CapsReader } from "../api/caps";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";
import { useCaps } from "./useCaps";
import type { PolledReader } from "./usePolledReader";
import { usePolledReader } from "./usePolledReader";

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
  readonly focusReader: PolledReader<readonly Focus[]>;
  readonly proposalReader: PolledReader<readonly Proposal[]>;
  readonly capsReader: CapsReader;
}

export function useAppState({ focusReader, proposalReader, capsReader }: AppStateDeps): AppState {
  const focuses = usePolledReader(focusReader);
  const proposals = usePolledReader(proposalReader);
  const caps = useCaps(capsReader);

  if (focuses.status === "error") return { status: "error", error: focuses.error, caps };
  if (proposals.status === "error") return { status: "error", error: proposals.error, caps };
  if (focuses.status === "loading" || proposals.status === "loading") {
    return { status: "loading", caps };
  }
  return {
    status: "ready",
    focuses: focuses.value,
    proposals: proposals.value,
    caps,
  };
}
