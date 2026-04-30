import type { Proposal } from "../types/proposal";
import type { Unsubscribe } from "./focuses";
import type { ProposalReader } from "./proposals";

export function createFixtureProposalReader(proposals: readonly Proposal[]): ProposalReader {
  return {
    list: () => Promise.resolve(proposals),
  };
}

export function createMutableProposalReader(initial: readonly Proposal[]): {
  reader: ProposalReader;
  setProposals(next: readonly Proposal[]): void;
} {
  let current = [...initial];
  let listener: (() => void) | null = null;
  const reader: ProposalReader = {
    list: () => Promise.resolve(current),
    subscribe(onChange): Unsubscribe {
      listener = onChange;
      return () => {
        listener = null;
      };
    },
  };
  return {
    reader,
    setProposals(next) {
      current = [...next];
      listener?.();
    },
  };
}
