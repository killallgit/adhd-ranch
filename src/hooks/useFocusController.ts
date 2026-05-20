import { useMemo } from "react";
import type { FocusWriter, WriteOutcome } from "../api/focusWriter";
import type { TimerPreset } from "../types/timer";

export type ReportWriteFailure = (op: string, outcome: WriteOutcome) => void;

export interface FocusController {
  readonly deleteFocus: (focusId: string) => Promise<WriteOutcome>;
  readonly duplicateFocus: (focusId: string) => Promise<WriteOutcome>;
  readonly renameFocus: (focusId: string, title: string) => Promise<WriteOutcome>;
  readonly appendTask: (focusId: string, text: string) => Promise<WriteOutcome>;
  readonly deleteTask: (focusId: string, index: number) => Promise<WriteOutcome>;
  readonly updateTask: (focusId: string, index: number, text: string) => Promise<WriteOutcome>;
  readonly toggleTask: (focusId: string, index: number, done: boolean) => Promise<WriteOutcome>;
  readonly startTimer: (focusId: string, preset: TimerPreset) => Promise<WriteOutcome>;
  readonly clearTimer: (focusId: string) => Promise<WriteOutcome>;
  readonly startTaskTimer: (
    focusId: string,
    index: number,
    preset: TimerPreset,
  ) => Promise<WriteOutcome>;
  readonly clearTaskTimer: (focusId: string, index: number) => Promise<WriteOutcome>;
}

export function useFocusController(
  focusWriter: FocusWriter,
  onWriteFailure: ReportWriteFailure,
): FocusController {
  return useMemo(() => {
    async function run(op: string, write: () => Promise<WriteOutcome>): Promise<WriteOutcome> {
      let outcome: WriteOutcome;
      try {
        outcome = await write();
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        outcome = { ok: false, kind: "ipc", message };
      }
      onWriteFailure(op, outcome);
      return outcome;
    }

    return {
      deleteFocus(focusId: string) {
        return run("delete_focus", () => focusWriter.deleteFocus(focusId));
      },
      duplicateFocus(focusId: string) {
        return run("duplicate_focus", () => focusWriter.duplicateFocus(focusId));
      },
      renameFocus(focusId: string, title: string) {
        return run("rename_focus", () => focusWriter.renameFocus(focusId, title));
      },
      appendTask(focusId: string, text: string) {
        return run("append_task", () => focusWriter.appendTask(focusId, text));
      },
      deleteTask(focusId: string, index: number) {
        return run("delete_task", () => focusWriter.deleteTask(focusId, index));
      },
      updateTask(focusId: string, index: number, text: string) {
        return run("update_task", () => focusWriter.updateTask(focusId, index, text));
      },
      toggleTask(focusId: string, index: number, done: boolean) {
        return run("toggle_task", () => focusWriter.toggleTask(focusId, index, done));
      },
      startTimer(focusId: string, preset: TimerPreset) {
        return run("start_timer", () => focusWriter.startTimer(focusId, preset));
      },
      clearTimer(focusId: string) {
        return run("clear_timer", () => focusWriter.clearTimer(focusId));
      },
      startTaskTimer(focusId: string, index: number, preset: TimerPreset) {
        return run("start_task_timer", () => focusWriter.startTaskTimer(focusId, index, preset));
      },
      clearTaskTimer(focusId: string, index: number) {
        return run("clear_task_timer", () => focusWriter.clearTaskTimer(focusId, index));
      },
    };
  }, [focusWriter, onWriteFailure]);
}
