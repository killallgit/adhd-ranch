import React from "react";
import ReactDOM from "react-dom/client";
import { createHookFiringReader, createHookWiringReader } from "./api/agentDebug";
import { createTauriAgentSessionReader } from "./api/tauriAgentSessionReader";
import { AgentDebugWindow } from "./components/AgentDebugWindow";
import { usePolledReader } from "./hooks/usePolledReader";
import type { AgentSession } from "./types/agentSession";
import type { HookFiring } from "./types/generated/HookFiring";
import "./styles.css";

const wiringReader = createHookWiringReader();
const firingReader = createHookFiringReader();
const sessionReader = createTauriAgentSessionReader();

const NO_SESSIONS: readonly AgentSession[] = [];
const NO_FIRINGS: readonly HookFiring[] = [];

function AgentDebugApp() {
  const wiring = usePolledReader(wiringReader);
  const firings = usePolledReader(firingReader);
  const sessions = usePolledReader(sessionReader);

  return (
    <AgentDebugWindow
      wiring={wiring.status === "ready" ? wiring.value : null}
      sessions={sessions.status === "ready" ? sessions.value : NO_SESSIONS}
      firings={firings.status === "ready" ? firings.value : NO_FIRINGS}
    />
  );
}

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <AgentDebugApp />
  </React.StrictMode>,
);
