import type { PolledReader } from "../hooks/usePolledReader";
import type { Focus } from "../types/focus";

export function createFixtureFocusReader(
  focuses: readonly Focus[],
): PolledReader<readonly Focus[]> {
  return {
    read: () => Promise.resolve(focuses),
  };
}
