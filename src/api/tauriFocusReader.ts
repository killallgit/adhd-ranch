import type { Focus } from "../types/focus";
import type { FocusTimer } from "../types/generated/FocusTimer";
import type { PolledReader } from "./polledReader";
import { createTauriReader } from "./tauriReader";

interface RustFocus {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly created_at: string;
  readonly tasks: readonly {
    id: string;
    text: string;
    done?: boolean;
    timer?: FocusTimer | null;
  }[];
  readonly timer?: FocusTimer | null;
}

function fromRust(raw: RustFocus): Focus {
  return {
    id: raw.id,
    title: raw.title,
    description: raw.description,
    created_at: raw.created_at,
    tasks: raw.tasks.map((t) => ({
      id: t.id,
      text: t.text,
      done: t.done ?? false,
      timer: t.timer ?? null,
    })),
    timer: raw.timer ?? null,
  };
}

export function createTauriFocusReader(): PolledReader<readonly Focus[]> {
  return createTauriReader<readonly RustFocus[], readonly Focus[]>({
    invokeKey: "list_focuses",
    eventKey: "focuses-changed",
    map: (raw) => raw.map(fromRust),
  });
}
