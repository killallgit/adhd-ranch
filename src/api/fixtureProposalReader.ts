import type { Proposal } from "../types/proposal";
import type { ProposalReader } from "./proposals";

export function createFixtureProposalReader(proposals: readonly Proposal[]): ProposalReader {
  return {
    list: () => Promise.resolve(proposals),
  };
}
