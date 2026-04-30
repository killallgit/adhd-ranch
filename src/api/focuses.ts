import type { Focus } from "../types/focus";

export type Unsubscribe = () => void;

export interface FocusReader {
  list(): Promise<readonly Focus[]>;
  subscribe?(onChange: () => void): Unsubscribe | Promise<Unsubscribe>;
}
