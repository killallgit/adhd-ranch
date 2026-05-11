import { type Caps, DEFAULT_CAPS } from "../api/caps";
import type { PolledReader } from "../api/polledReader";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";
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
  readonly capsReader: PolledReader<Caps>;
}

export function useAppState({ focusReader, proposalReader, capsReader }: AppStateDeps): AppState {
  const focuses = usePolledReader(focusReader);
  const proposals = usePolledReader(proposalReader);
  const capsState = usePolledReader(capsReader);
  const caps = capsState.status === "ready" ? capsState.value : DEFAULT_CAPS;

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
