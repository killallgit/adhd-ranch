import type { FocusReader } from "../api/focuses";
import { useFocuses } from "../hooks/useFocuses";
import { FocusList } from "./FocusList";

export interface AppProps {
  readonly focusReader: FocusReader;
}

export function App({ focusReader }: AppProps) {
  const state = useFocuses(focusReader);

  return (
    <div data-testid="app-root" className="app-root">
      <header className="app-header">
        <h1 className="app-title">adhd-ranch</h1>
      </header>
      <main className="app-body">
        {state.status === "loading" && <p data-testid="app-loading">Loading…</p>}
        {state.status === "error" && (
          <p data-testid="app-error" role="alert">
            {state.error.message}
          </p>
        )}
        {state.status === "ready" && <FocusList focuses={state.focuses} />}
      </main>
    </div>
  );
}
