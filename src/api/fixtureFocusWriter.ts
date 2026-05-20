import type { FocusWriter, WriteOutcome } from "./focusWriter";

export interface FixtureFocusWriterOptions {
  readonly failure?: Extract<WriteOutcome, { ok: false }>;
}

export function createFixtureFocusWriter(opts: FixtureFocusWriterOptions = {}): FocusWriter {
  const outcome: WriteOutcome = opts.failure ?? { ok: true };
  const result = () => Promise.resolve(outcome);
  return {
    createFocus: result,
    duplicateFocus: result,
    deleteFocus: result,
    renameFocus: result,
    appendTask: result,
    deleteTask: result,
    updateTask: result,
    toggleTask: result,
    startTimer: result,
    clearTimer: result,
    startTaskTimer: result,
    clearTaskTimer: result,
  };
}
