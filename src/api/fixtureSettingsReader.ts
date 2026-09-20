import type { Settings } from "../types/settings";
import type { PolledReader } from "./polledReader";

// Not the Rust defaults: `agents.enabled` is off there, and every test that renders
// the ranch wants agent animals in it.
const FIXTURE: Settings = {
  caps: { max_focuses: 5, max_tasks_per_focus: 7 },
  notifications: { sources: {} },
  widget: { always_on_top: false, confirm_delete: true },
  displays: { enabled_indices: [0] },
  agents: { enabled: true },
  pens: { max_size: 320 },
};

export function createFixtureSettingsReader(
  overrides: Partial<Settings> = {},
): PolledReader<Settings> {
  return {
    read: () => Promise.resolve({ ...FIXTURE, ...overrides }),
  };
}
