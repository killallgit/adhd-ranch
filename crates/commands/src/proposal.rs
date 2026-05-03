use adhd_ranch_domain::{NewFocus, Proposal, ProposalId, ProposalKind};
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
                    timer_preset: None,
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
        self.lifecycle.accept(id, edit)
    }

    pub fn reject_proposal(&self, id: &str) -> Result<DecisionOutcome, CommandError> {
        self.lifecycle.reject(id)
    }
}
