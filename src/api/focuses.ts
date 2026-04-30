import type { Focus } from "../types/focus";

export interface FocusReader {
  list(): Promise<readonly Focus[]>;
}
