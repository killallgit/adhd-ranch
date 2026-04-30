import type { NewFocus, Proposal } from "../types/proposal";
import type { Unsubscribe } from "./focuses";

export interface ProposalDecisionResult {
  readonly id: string;
  readonly target: string | null;
}

export interface ProposalReader {
  list(): Promise<readonly Proposal[]>;
  subscribe?(onChange: () => void): Unsubscribe | Promise<Unsubscribe>;
}

export interface ProposalEdit {
  readonly target_focus_id?: string;
  readonly task_text?: string;
  readonly new_focus?: NewFocus;
}

export interface ProposalWriter {
  accept(id: string, edit?: ProposalEdit): Promise<ProposalDecisionResult>;
  reject(id: string): Promise<ProposalDecisionResult>;
}
