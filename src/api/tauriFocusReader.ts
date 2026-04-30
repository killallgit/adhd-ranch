import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Focus } from "../types/focus";
import type { FocusReader, Unsubscribe } from "./focuses";

interface RustFocus {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly created_at: string;
  readonly tasks: readonly { id: string; text: string }[];
}

const FOCUSES_CHANGED = "focuses-changed";

function fromRust(raw: RustFocus): Focus {
  return {
    id: raw.id,
    title: raw.title,
    description: raw.description,
    tasks: raw.tasks.map((t) => ({ id: t.id, text: t.text })),
  };
}

export function createTauriFocusReader(): FocusReader {
  return {
    async list() {
      const raw = await invoke<RustFocus[]>("list_focuses");
      return raw.map(fromRust);
    },
    async subscribe(onChange): Promise<Unsubscribe> {
      const unlisten = await listen(FOCUSES_CHANGED, () => {
        onChange();
      });
      return () => {
        unlisten();
      };
    },
  };
}
