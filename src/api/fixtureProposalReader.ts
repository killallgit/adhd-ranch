import type { Proposal } from "../types/proposal";
import type { PolledReader } from "./polledReader";

export function createFixtureProposalReader(
  proposals: readonly Proposal[],
): PolledReader<readonly Proposal[]> {
  return {
    read: () => Promise.resolve(proposals),
  };
}
