import { useState } from "react";
import type { ProposalWriter } from "../api/proposals";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";

export interface PendingTrayProps {
  readonly proposals: readonly Proposal[];
  readonly focuses: readonly Focus[];
  readonly proposalWriter: ProposalWriter;
}

export function PendingTray({ proposals, focuses, proposalWriter }: PendingTrayProps) {
  const [expanded, setExpanded] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [errorById, setErrorById] = useState<Record<string, string>>({});

  if (proposals.length === 0) return null;

  const handle = async (id: string, action: "accept" | "reject") => {
    setBusyId(id);
    setErrorById((prev) => ({ ...prev, [id]: "" }));
    try {
      if (action === "accept") await proposalWriter.accept(id);
      else await proposalWriter.reject(id);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setErrorById((prev) => ({ ...prev, [id]: message }));
    } finally {
      setBusyId((prev) => (prev === id ? null : prev));
    }
  };

  return (
    <section
      data-testid="pending-tray"
      data-expanded={expanded}
      className="pending-tray"
      aria-label="pending proposals"
    >
      <button
        type="button"
        className="pending-tray-toggle"
        aria-expanded={expanded}
        aria-controls="pending-tray-list"
        onClick={() => setExpanded((prev) => !prev)}
      >
        <span aria-hidden="true">📥</span>
        <span data-testid="pending-tray-count">{proposals.length}</span> pending
      </button>
      {expanded && (
        <ul id="pending-tray-list" className="pending-tray-list">
          {proposals.map((proposal) => (
            <ProposalCard
              key={proposal.id}
              proposal={proposal}
              focuses={focuses}
              busy={busyId === proposal.id}
              error={errorById[proposal.id] ?? ""}
              onAccept={() => handle(proposal.id, "accept")}
              onReject={() => handle(proposal.id, "reject")}
            />
          ))}
        </ul>
      )}
    </section>
  );
}

interface ProposalCardProps {
  readonly proposal: Proposal;
  readonly focuses: readonly Focus[];
  readonly busy: boolean;
  readonly error: string;
  readonly onAccept: () => void;
  readonly onReject: () => void;
}

function ProposalCard({ proposal, focuses, busy, error, onAccept, onReject }: ProposalCardProps) {
  const [reasoningOpen, setReasoningOpen] = useState(false);
  const target = describeTarget(proposal, focuses);

  return (
    <li data-testid="proposal-card" className="proposal-card">
      <p className="proposal-summary">{proposal.summary}</p>
      {target && (
        <p data-testid="proposal-target" className="proposal-target">
          → {target}
        </p>
      )}
      <div className="proposal-actions">
        <button
          type="button"
          className="proposal-accept"
          aria-label={`accept proposal ${proposal.id}`}
          disabled={busy}
          onClick={onAccept}
        >
          ✓
        </button>
        <button
          type="button"
          className="proposal-reject"
          aria-label={`reject proposal ${proposal.id}`}
          disabled={busy}
          onClick={onReject}
        >
          ✗
        </button>
        <button
          type="button"
          className="proposal-reasoning-toggle"
          aria-expanded={reasoningOpen}
          aria-label={`toggle reasoning for ${proposal.id}`}
          onClick={() => setReasoningOpen((prev) => !prev)}
        >
          ?
        </button>
      </div>
      {reasoningOpen && (
        <p data-testid="proposal-reasoning" className="proposal-reasoning">
          {proposal.reasoning}
        </p>
      )}
      {error && (
        <p data-testid="proposal-error" role="alert" className="proposal-error">
          {error}
        </p>
      )}
    </li>
  );
}

function describeTarget(proposal: Proposal, focuses: readonly Focus[]): string | null {
  switch (proposal.kind) {
    case "add_task": {
      const focus = focuses.find((f) => f.id === proposal.target_focus_id);
      return focus ? `Add to "${focus.title}"` : `Add to ${proposal.target_focus_id}`;
    }
    case "new_focus":
      return `New focus: "${proposal.new_focus.title}"`;
    case "discard":
      return "Discard";
  }
}
