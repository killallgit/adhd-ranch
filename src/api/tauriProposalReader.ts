import { invoke } from "@tauri-apps/api/core";
import type { PolledReader } from "../hooks/usePolledReader";
import type { CommandError } from "../types/error";
import type { Proposal } from "../types/proposal";
import type { ProposalDecisionResult, ProposalEdit, ProposalWriter } from "./proposals";
import { createTauriReader } from "./tauriReader";

interface RustDecisionResponse {
  readonly id: string;
  readonly target: string | null;
}

export function createTauriProposalReader(): PolledReader<readonly Proposal[]> {
  return createTauriReader<readonly Proposal[], readonly Proposal[]>({
    invokeKey: "list_proposals",
    eventKey: "proposals-changed",
    map: (raw) => raw,
  });
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
