import type { Proposal } from "../types/proposal";
import type { Unsubscribe } from "./focuses";

export interface ProposalReader {
  list(): Promise<readonly Proposal[]>;
  subscribe?(onChange: () => void): Unsubscribe | Promise<Unsubscribe>;
}
