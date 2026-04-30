import type { Focus } from "../types/focus";
import type { FocusReader } from "./focuses";

export function createFixtureFocusReader(focuses: readonly Focus[]): FocusReader {
  return {
    list: () => Promise.resolve(focuses),
  };
}
