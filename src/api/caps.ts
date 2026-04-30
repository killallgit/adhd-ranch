import { invoke } from "@tauri-apps/api/core";

export interface Caps {
  readonly max_focuses: number;
  readonly max_tasks_per_focus: number;
}

export interface CapsReader {
  get(): Promise<Caps>;
}

export const DEFAULT_CAPS: Caps = {
  max_focuses: 5,
  max_tasks_per_focus: 7,
};

export function createTauriCapsReader(): CapsReader {
  return {
    get: () => invoke<Caps>("get_caps"),
  };
}

export function createFixtureCapsReader(caps: Caps): CapsReader {
  return {
    get: () => Promise.resolve(caps),
  };
}
