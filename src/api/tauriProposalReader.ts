import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Proposal } from "../types/proposal";
import type { CommandError } from "../types/error";
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
      const result = await invoke<RustDecisionResponse>("accept_proposal", { id, edit }).catch(
        (e: unknown): never => {
          console.error("[adhd-ranch] accept_proposal", e as CommandError);
          throw e;
        },
      );
      return { id: result.id, target: result.target };
    },
    async reject(id: string): Promise<ProposalDecisionResult> {
      const result = await invoke<RustDecisionResponse>("reject_proposal", { id }).catch(
        (e: unknown): never => {
          console.error("[adhd-ranch] reject_proposal", e as CommandError);
          throw e;
        },
      );
      return { id: result.id, target: result.target };
    },
  };
}
