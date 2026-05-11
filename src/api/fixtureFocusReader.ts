import type { Focus } from "../types/focus";
import type { PolledReader } from "./polledReader";

export function createFixtureFocusReader(
  focuses: readonly Focus[],
): PolledReader<readonly Focus[]> {
  return {
    read: () => Promise.resolve(focuses),
  };
}
