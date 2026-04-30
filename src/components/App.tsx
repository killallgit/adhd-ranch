import { useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import type { FocusReader } from "../api/focuses";
import type { ProposalReader, ProposalWriter } from "../api/proposals";
import { useFocuses } from "../hooks/useFocuses";
import { useProposals } from "../hooks/useProposals";
import { FocusList } from "./FocusList";
import { NewFocusForm } from "./NewFocusForm";
import { PendingTray } from "./PendingTray";

export interface AppProps {
  readonly focusReader: FocusReader;
  readonly focusWriter: FocusWriter;
  readonly proposalReader: ProposalReader;
  readonly proposalWriter: ProposalWriter;
}

export function App({ focusReader, focusWriter, proposalReader, proposalWriter }: AppProps) {
  const focuses = useFocuses(focusReader);
  const proposals = useProposals(proposalReader);
  const [busyFocusId, setBusyFocusId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const focusList = focuses.status === "ready" ? focuses.focuses : [];
  const proposalList = proposals.status === "ready" ? proposals.proposals : [];

  const wrap = async (focusId: string, run: () => Promise<unknown>) => {
    setBusyFocusId(focusId);
    setError(null);
    try {
      await run();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyFocusId((prev) => (prev === focusId ? null : prev));
    }
  };

  const handleClearTask = (focusId: string, index: number) =>
    wrap(focusId, () => focusWriter.deleteTask(focusId, index));

  const handleDeleteFocus = (focusId: string) =>
    wrap(focusId, () => focusWriter.deleteFocus(focusId));

  const handleCreateFocus = async (input: { title: string; description: string }) => {
    await focusWriter.createFocus(input);
  };

  return (
    <div data-testid="app-root" className="app-root">
      <header className="app-header">
        <h1 className="app-title">adhd-ranch</h1>
      </header>
      <main className="app-body">
        {focuses.status === "loading" && <p data-testid="app-loading">Loading…</p>}
        {focuses.status === "error" && (
          <p data-testid="app-error" role="alert">
            {focuses.error.message}
          </p>
        )}
        {focuses.status === "ready" && (
          <FocusList
            focuses={focuses.focuses}
            busyFocusId={busyFocusId}
            onClearTask={handleClearTask}
            onDeleteFocus={handleDeleteFocus}
          />
        )}
        <NewFocusForm onCreate={handleCreateFocus} />
        {error && (
          <p data-testid="app-action-error" role="alert">
            {error}
          </p>
        )}
      </main>
      <footer className="app-footer">
        <PendingTray proposals={proposalList} focuses={focusList} proposalWriter={proposalWriter} />
      </footer>
    </div>
  );
}
