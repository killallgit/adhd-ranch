import type { AgentSession } from "../types/agentSession";
import type { PolledReader } from "./polledReader";
import { createTauriReader } from "./tauriReader";

export function createTauriAgentSessionReader(): PolledReader<readonly AgentSession[]> {
  return createTauriReader<readonly AgentSession[], readonly AgentSession[]>({
    invokeKey: "list_agent_sessions",
    eventKey: "agent-sessions-changed",
    map: (raw) => raw,
  });
}
