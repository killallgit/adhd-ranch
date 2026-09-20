import React from "react";
import ReactDOM from "react-dom/client";
import { createHookFiringReader, createHookWiringReader } from "./api/agentDebug";
import { createTauriAgentSessionReader } from "./api/tauriAgentSessionReader";
import { AgentDebugWindow } from "./components/AgentDebugWindow";
import { usePolledReader } from "./hooks/usePolledReader";
import "./styles.css";

const wiringReader = createHookWiringReader();
const firingReader = createHookFiringReader();
const sessionReader = createTauriAgentSessionReader();

function AgentDebugApp() {
  const wiring = usePolledReader(wiringReader);
  const firings = usePolledReader(firingReader);
  const sessions = usePolledReader(sessionReader);

  return <AgentDebugWindow wiring={wiring} sessions={sessions} firings={firings} />;
}

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <AgentDebugApp />
  </React.StrictMode>,
);
