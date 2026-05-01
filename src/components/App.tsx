import { getCurrentWindow } from "@tauri-apps/api/window";
import { useState } from "react";
import type { CapsReader } from "../api/caps";
import type { FocusWriter } from "../api/focusWriter";
import type { FocusReader } from "../api/focuses";
import type { ProposalReader, ProposalWriter } from "../api/proposals";
import { useAppState } from "../hooks/useAppState";
import { computeCapState } from "../lib/capState";
import { CapBadge } from "./CapBadge";
import { FocusList } from "./FocusList";
import { NewFocusForm } from "./NewFocusForm";
import { PendingTray } from "./PendingTray";
import { Titlebar } from "./Titlebar";

export interface AppProps {
  readonly focusReader: FocusReader;
  readonly focusWriter: FocusWriter;
  readonly proposalReader: ProposalReader;
  readonly proposalWriter: ProposalWriter;
  readonly capsReader: CapsReader;
}

export function App({
  focusReader,
  focusWriter,
  proposalReader,
  proposalWriter,
  capsReader,
}: AppProps) {
  const state = useAppState({ focusReader, proposalReader, capsReader });
  const [busyFocusId, setBusyFocusId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const focusList = state.status === "ready" ? state.focuses : [];
  const proposalList = state.status === "ready" ? state.proposals : [];
  const capState = computeCapState(focusList, state.caps);

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

  const handleClose = () => {
    void getCurrentWindow().hide();
  };

  return (
    <div data-testid="app-root" className="app-root" data-over-cap={capState.anyOver}>
      <Titlebar onClose={handleClose} />
      <header className="app-header">
        <h1 className="app-title">adhd-ranch</h1>
        <CapBadge capState={capState} maxFocuses={state.caps.max_focuses} />
      </header>
      <main className="app-body">
        {state.status === "loading" && <p data-testid="app-loading">Loading…</p>}
        {state.status === "error" && (
          <p data-testid="app-error" role="alert">
            {state.error.message}
          </p>
        )}
        {state.status === "ready" && (
          <FocusList
            focuses={state.focuses}
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
