import type { PolledReader } from "../hooks/usePolledReader";
import type { Proposal } from "../types/proposal";

export function createFixtureProposalReader(
  proposals: readonly Proposal[],
): PolledReader<readonly Proposal[]> {
  return {
    read: () => Promise.resolve(proposals),
  };
}
