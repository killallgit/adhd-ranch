import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PolledReader, Unsubscribe } from "./polledReader";

export interface TauriReaderConfig<Raw, Out> {
  readonly invokeKey: string;
  readonly eventKey?: string;
  readonly map: (raw: Raw) => Out;
}

export function createTauriReader<Raw, Out>(cfg: TauriReaderConfig<Raw, Out>): PolledReader<Out> {
  const { invokeKey, eventKey, map } = cfg;
  const read = () => invoke<Raw>(invokeKey).then(map);
  if (!eventKey) return { read };
  return {
    read,
    subscribe: async (onChange): Promise<Unsubscribe> => {
      const un = await listen(eventKey, () => onChange());
      return () => {
        un();
      };
    },
  };
}
