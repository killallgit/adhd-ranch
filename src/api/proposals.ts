import type { NewFocus, Proposal } from "../types/proposal";

export interface ProposalDecisionResult {
  readonly id: string;
  readonly target: string | null;
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

export type { Proposal };
