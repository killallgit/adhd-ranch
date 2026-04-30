import type { Proposal } from "../types/proposal";
import type { Unsubscribe } from "./focuses";

export interface ProposalDecisionResult {
  readonly id: string;
  readonly target: string | null;
}

export interface ProposalReader {
  list(): Promise<readonly Proposal[]>;
  subscribe?(onChange: () => void): Unsubscribe | Promise<Unsubscribe>;
}

export interface ProposalWriter {
  accept(id: string): Promise<ProposalDecisionResult>;
  reject(id: string): Promise<ProposalDecisionResult>;
}
