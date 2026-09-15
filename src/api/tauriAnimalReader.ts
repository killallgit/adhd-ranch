import type { Animal } from "../types/animal";
import type { PolledReader } from "./polledReader";
import { createTauriReader } from "./tauriReader";

export function createTauriAnimalReader(): PolledReader<readonly Animal[]> {
  return createTauriReader<readonly Animal[], readonly Animal[]>({
    invokeKey: "list_animals",
    eventKey: "animals-changed",
    map: (raw) => raw,
  });
}
