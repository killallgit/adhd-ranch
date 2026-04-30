import { useState } from "react";
import type { ProposalEdit } from "../api/proposals";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";

export interface EditProposalModalProps {
  readonly proposal: Proposal;
  readonly focuses: readonly Focus[];
  readonly onConfirm: (edit: ProposalEdit) => void;
  readonly onCancel: () => void;
}

export function EditProposalModal({
  proposal,
  focuses,
  onConfirm,
  onCancel,
}: EditProposalModalProps) {
  return (
    <div className="modal-backdrop" data-testid="edit-proposal-backdrop">
      <dialog open aria-modal="true" aria-label="edit proposal" className="modal">
        <h3 className="modal-title">Edit proposal</h3>
        {proposal.kind === "add_task" && (
          <AddTaskEditor
            proposal={proposal}
            focuses={focuses}
            onConfirm={onConfirm}
            onCancel={onCancel}
          />
        )}
        {proposal.kind === "new_focus" && (
          <NewFocusEditor proposal={proposal} onConfirm={onConfirm} onCancel={onCancel} />
        )}
        {proposal.kind === "discard" && (
          <DiscardEditor onConfirm={() => onConfirm({})} onCancel={onCancel} />
        )}
      </dialog>
    </div>
  );
}

interface CommonEditorProps {
  readonly onConfirm: (edit: ProposalEdit) => void;
  readonly onCancel: () => void;
}

interface AddTaskEditorProps extends CommonEditorProps {
  readonly proposal: Extract<Proposal, { kind: "add_task" }>;
  readonly focuses: readonly Focus[];
}

function AddTaskEditor({ proposal, focuses, onConfirm, onCancel }: AddTaskEditorProps) {
  const [targetId, setTargetId] = useState(proposal.target_focus_id);
  const [text, setText] = useState(proposal.task_text);

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    onConfirm({
      target_focus_id: targetId,
      task_text: text,
    });
  };

  return (
    <form onSubmit={submit} data-testid="edit-add-task-form">
      <label className="modal-field">
        <span>Target focus</span>
        <select
          aria-label="target focus"
          value={targetId}
          onChange={(e) => setTargetId(e.target.value)}
        >
          {focuses.map((f) => (
            <option key={f.id} value={f.id}>
              {f.title}
            </option>
          ))}
        </select>
      </label>
      <label className="modal-field">
        <span>Task text</span>
        <input
          type="text"
          aria-label="task text"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </label>
      <ModalActions onCancel={onCancel} />
    </form>
  );
}

interface NewFocusEditorProps extends CommonEditorProps {
  readonly proposal: Extract<Proposal, { kind: "new_focus" }>;
}

function NewFocusEditor({ proposal, onConfirm, onCancel }: NewFocusEditorProps) {
  const [title, setTitle] = useState(proposal.new_focus.title);
  const [description, setDescription] = useState(proposal.new_focus.description);

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    onConfirm({ new_focus: { title, description } });
  };

  return (
    <form onSubmit={submit} data-testid="edit-new-focus-form">
      <label className="modal-field">
        <span>Title</span>
        <input
          type="text"
          aria-label="new focus title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
        />
      </label>
      <label className="modal-field">
        <span>Description</span>
        <input
          type="text"
          aria-label="new focus description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />
      </label>
      <ModalActions onCancel={onCancel} />
    </form>
  );
}

function DiscardEditor({ onConfirm, onCancel }: { onConfirm: () => void; onCancel: () => void }) {
  return (
    <div data-testid="edit-discard-form">
      <p>Discard proposals carry no editable fields.</p>
      <div className="modal-actions">
        <button type="button" onClick={onConfirm}>
          Accept anyway
        </button>
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}

function ModalActions({ onCancel }: { onCancel: () => void }) {
  return (
    <div className="modal-actions">
      <button type="submit">Accept (edited)</button>
      <button type="button" onClick={onCancel}>
        Cancel
      </button>
    </div>
  );
}
