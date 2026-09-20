import type { HookFiring } from "../types/generated/HookFiring";
import type { HookWiring } from "../types/generated/HookWiring";
import type { PolledReader } from "./polledReader";
import { createTauriReader } from "./tauriReader";

export function createHookFiringReader(): PolledReader<readonly HookFiring[]> {
  return createTauriReader<readonly HookFiring[], readonly HookFiring[]>({
    invokeKey: "list_hook_firings",
    eventKey: "agent-hook-fired",
    map: (raw) => raw,
  });
}

// Follows session changes rather than firings: what it reports only moves when the
// agents setting is toggled, and that is emitted on the same event.
export function createHookWiringReader(): PolledReader<HookWiring> {
  return createTauriReader<HookWiring, HookWiring>({
    invokeKey: "agent_wiring",
    eventKey: "agent-sessions-changed",
    map: (raw) => raw,
  });
}
