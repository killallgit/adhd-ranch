import type { AgentSession } from "./agentSession";
import type { Focus } from "./focus";

interface AnimalBase {
  readonly id: string;
  readonly name: string;
  readonly expired: boolean;
  readonly scale: number;
}

export interface FocusAnimal extends AnimalBase {
  readonly kind: "focus";
  readonly focus: Focus;
}

interface AgentAnimal extends AnimalBase {
  readonly kind: "agent";
  readonly session: AgentSession;
}

export type Animal = FocusAnimal | AgentAnimal;
