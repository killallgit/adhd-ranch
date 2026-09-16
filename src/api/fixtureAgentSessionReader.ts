import type { AgentSession } from "../types/agentSession";
import type { PolledReader } from "./polledReader";

export function createFixtureAgentSessionReader(
  sessions: readonly AgentSession[],
): PolledReader<readonly AgentSession[]> {
  return {
    read: () => Promise.resolve(sessions),
  };
}
