import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Proposal } from "../types/proposal";
import type { Unsubscribe } from "./focuses";
import type {
  ProposalDecisionResult,
  ProposalEdit,
  ProposalReader,
  ProposalWriter,
} from "./proposals";

const PROPOSALS_CHANGED = "proposals-changed";

interface RustDecisionResponse {
  readonly id: string;
  readonly target: string | null;
}

export function createTauriProposalReader(): ProposalReader {
  return {
    list: () => invoke<Proposal[]>("list_proposals"),
    async subscribe(onChange): Promise<Unsubscribe> {
      const unlisten = await listen(PROPOSALS_CHANGED, () => {
        onChange();
      });
      return () => {
        unlisten();
      };
    },
  };
}

export function createTauriProposalWriter(): ProposalWriter {
  return {
    async accept(id: string, edit?: ProposalEdit): Promise<ProposalDecisionResult> {
      const result = await invoke<RustDecisionResponse>("accept_proposal", { id, edit });
      return { id: result.id, target: result.target };
    },
    async reject(id: string): Promise<ProposalDecisionResult> {
      const result = await invoke<RustDecisionResponse>("reject_proposal", { id });
      return { id: result.id, target: result.target };
    },
  };
}
