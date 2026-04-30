export interface NewFocus {
  readonly title: string;
  readonly description: string;
}

export type ProposalKind =
  | { readonly kind: "add_task"; readonly target_focus_id: string; readonly task_text: string }
  | { readonly kind: "new_focus"; readonly new_focus: NewFocus }
  | { readonly kind: "discard" };

export type Proposal = ProposalKind & {
  readonly id: string;
  readonly summary: string;
  readonly reasoning: string;
  readonly created_at: string;
};
