use adhd_ranch_domain::{Decision, DecisionKind, NewFocus, Proposal, ProposalId, ProposalKind};
use serde::{Deserialize, Serialize};

use crate::error::CommandError;
use crate::Commands;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProposalInput {
    pub kind: String,
    #[serde(default)]
    pub target_focus_id: Option<String>,
    #[serde(default)]
    pub task_text: Option<String>,
    #[serde(default)]
    pub new_focus: Option<NewFocus>,
    pub summary: String,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatedProposal {
    pub id: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ProposalEdit {
    #[serde(default)]
    pub target_focus_id: Option<String>,
    #[serde(default)]
    pub task_text: Option<String>,
    #[serde(default)]
    pub new_focus: Option<NewFocus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionOutcome {
    pub id: String,
    pub target: Option<String>,
}

impl Commands {
    pub fn list_proposals(&self) -> Result<Vec<Proposal>, CommandError> {
        Ok(self.queue.list()?)
    }

    pub fn create_proposal(
        &self,
        input: CreateProposalInput,
    ) -> Result<CreatedProposal, CommandError> {
        let kind = match input.kind.as_str() {
            "add_task" => ProposalKind::AddTask {
                target_focus_id: input.target_focus_id.clone().unwrap_or_default(),
                task_text: input.task_text.clone().unwrap_or_default(),
            },
            "new_focus" => ProposalKind::NewFocus {
                new_focus: input.new_focus.clone().unwrap_or(NewFocus {
                    title: String::new(),
                    description: String::new(),
                }),
            },
            "discard" => ProposalKind::Discard,
            other => {
                return Err(CommandError::BadRequest(format!("unknown kind: {other}")));
            }
        };

        let id = (self.id_gen)();
        let proposal = Proposal {
            id: ProposalId(id.clone()),
            kind,
            summary: input.summary,
            reasoning: input.reasoning,
            created_at: (self.clock)(),
        };

        proposal.validate()?;
        self.queue.append(&proposal)?;
        Ok(CreatedProposal { id })
    }

    pub fn accept_proposal(
        &self,
        id: &str,
        edit: ProposalEdit,
    ) -> Result<DecisionOutcome, CommandError> {
        let original = self.load_proposal(id)?;
        let (proposal, edited) = apply_edit(original, &edit);
        proposal.validate()?;
        let outcome = self.dispatcher.apply(&proposal)?;
        self.record_decision(
            &proposal,
            DecisionKind::Accept,
            outcome.target.clone(),
            edited,
        )?;
        self.queue.remove(&proposal.id)?;
        Ok(DecisionOutcome {
            id: id.to_string(),
            target: outcome.target,
        })
    }

    pub fn reject_proposal(&self, id: &str) -> Result<DecisionOutcome, CommandError> {
        let proposal = self.load_proposal(id)?;
        self.record_decision(&proposal, DecisionKind::Reject, None, false)?;
        self.queue.remove(&proposal.id)?;
        Ok(DecisionOutcome {
            id: id.to_string(),
            target: None,
        })
    }

    fn load_proposal(&self, id: &str) -> Result<Proposal, CommandError> {
        self.queue
            .find(&ProposalId(id.to_string()))?
            .ok_or_else(|| CommandError::NotFound(format!("proposal not found: {id}")))
    }

    fn record_decision(
        &self,
        proposal: &Proposal,
        kind: DecisionKind,
        target: Option<String>,
        edited: bool,
    ) -> Result<(), CommandError> {
        let decision = Decision {
            ts: (self.clock)(),
            proposal_id: proposal.id.0.clone(),
            decision: kind,
            reasoning: proposal.reasoning.clone(),
            target,
            edited,
        };
        self.decisions.append(&decision)?;
        Ok(())
    }
}

fn apply_edit(mut proposal: Proposal, edit: &ProposalEdit) -> (Proposal, bool) {
    let mut edited = false;
    match &mut proposal.kind {
        ProposalKind::AddTask {
            target_focus_id,
            task_text,
        } => {
            if let Some(new_id) = edit.target_focus_id.as_ref() {
                if new_id != target_focus_id {
                    *target_focus_id = new_id.clone();
                    edited = true;
                }
            }
            if let Some(new_text) = edit.task_text.as_ref() {
                if new_text != task_text {
                    *task_text = new_text.clone();
                    edited = true;
                }
            }
        }
        ProposalKind::NewFocus { new_focus } => {
            if let Some(replacement) = edit.new_focus.as_ref() {
                if replacement != new_focus {
                    *new_focus = replacement.clone();
                    edited = true;
                }
            }
        }
        ProposalKind::Discard => {}
    }
    (proposal, edited)
}
