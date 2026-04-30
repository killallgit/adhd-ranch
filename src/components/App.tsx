import type { FocusReader } from "../api/focuses";
import type { ProposalReader, ProposalWriter } from "../api/proposals";
import { useFocuses } from "../hooks/useFocuses";
import { useProposals } from "../hooks/useProposals";
import { FocusList } from "./FocusList";
import { PendingTray } from "./PendingTray";

export interface AppProps {
  readonly focusReader: FocusReader;
  readonly proposalReader: ProposalReader;
  readonly proposalWriter: ProposalWriter;
}

export function App({ focusReader, proposalReader, proposalWriter }: AppProps) {
  const focuses = useFocuses(focusReader);
  const proposals = useProposals(proposalReader);

  const focusList = focuses.status === "ready" ? focuses.focuses : [];
  const proposalList = proposals.status === "ready" ? proposals.proposals : [];

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
        {focuses.status === "ready" && <FocusList focuses={focuses.focuses} />}
      </main>
      <footer className="app-footer">
        <PendingTray proposals={proposalList} focuses={focusList} proposalWriter={proposalWriter} />
      </footer>
    </div>
  );
}
