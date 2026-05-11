import type { PolledReader } from "../hooks/usePolledReader";
import { createTauriReader } from "./tauriReader";

export interface Caps {
  readonly max_focuses: number;
  readonly max_tasks_per_focus: number;
}

export const DEFAULT_CAPS: Caps = {
  max_focuses: 5,
  max_tasks_per_focus: 7,
};

export function createTauriCapsReader(): PolledReader<Caps> {
  return createTauriReader<Caps, Caps>({
    invokeKey: "get_caps",
    map: (raw) => raw,
  });
}

export function createFixtureCapsReader(caps: Caps): PolledReader<Caps> {
  return {
    read: () => Promise.resolve(caps),
  };
}
