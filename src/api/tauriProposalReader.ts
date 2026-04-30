import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Proposal } from "../types/proposal";
import type { Unsubscribe } from "./focuses";
import type { ProposalReader } from "./proposals";

const PROPOSALS_CHANGED = "proposals-changed";

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
