import type { AgentSession } from "./agentSession";
import type { Focus } from "./focus";

// The only thing a Focus animal and a Session animal have in common is that the
// ranch draws and moves them the same way. `resting` is that shared idea and nothing
// more: a Focus rests because its timer ran out, a Session rests because it is
// between turns. Neither knows why the other one does.
interface AnimalBase {
  readonly id: string;
  readonly name: string;
  readonly scale: number;
  readonly resting: boolean;
}

export interface FocusAnimal extends AnimalBase {
  readonly kind: "focus";
  readonly focus: Focus;
}

export interface SessionAnimal extends AnimalBase {
  readonly kind: "agent";
  readonly session: AgentSession;
}

export type Animal = FocusAnimal | SessionAnimal;
