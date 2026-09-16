import { invoke } from "@tauri-apps/api/core";
import type { TimerOwner } from "../types/generated/TimerOwner";
import type { TimerPreset } from "../types/timer";

export const focusTimer = (focusId: string): TimerOwner => ({ kind: "focus", focus_id: focusId });

export const taskTimer = (focusId: string, index: number): TimerOwner => ({
  kind: "task",
  focus_id: focusId,
  index,
});

export type WriteOutcome =
  | { ok: true }
  | { ok: false; kind: "ipc" | "domain" | "not_found"; message: string };

interface CreateFocusInput {
  title: string;
  description?: string;
  timer_preset?: TimerPreset | null;
}

export interface FocusWriter {
  createFocus(input: CreateFocusInput): Promise<WriteOutcome>;
  duplicateFocus(focusId: string): Promise<WriteOutcome>;
  deleteFocus(focusId: string): Promise<WriteOutcome>;
  renameFocus(focusId: string, title: string): Promise<WriteOutcome>;
  appendTask(focusId: string, text: string): Promise<WriteOutcome>;
  deleteTask(focusId: string, index: number): Promise<WriteOutcome>;
  updateTask(focusId: string, index: number, text: string): Promise<WriteOutcome>;
  toggleTask(focusId: string, index: number, done: boolean): Promise<WriteOutcome>;
  startTimer(owner: TimerOwner, preset: TimerPreset): Promise<WriteOutcome>;
  clearTimer(owner: TimerOwner): Promise<WriteOutcome>;
}

function toFailure(e: unknown): WriteOutcome {
  if (e && typeof e === "object" && "type" in e && "message" in e) {
    const err = e as { type: string; message: string };
    const kind = err.type === "not_found" ? "not_found" : "domain";
    return { ok: false, kind, message: err.message };
  }
  const message = e instanceof Error ? e.message : String(e);
  return { ok: false, kind: "ipc", message };
}

async function runInvoke(cmd: string, args: Record<string, unknown>): Promise<WriteOutcome> {
  try {
    await invoke<unknown>(cmd, args);
    return { ok: true };
  } catch (e) {
    return toFailure(e);
  }
}

export function createTauriFocusWriter(): FocusWriter {
  return {
    createFocus({ title, description, timer_preset }) {
      return runInvoke("create_focus", {
        title,
        description,
        timerPreset: timer_preset ?? null,
      });
    },
    duplicateFocus(focusId) {
      return runInvoke("duplicate_focus", { focusId });
    },
    deleteFocus(focusId) {
      return runInvoke("delete_focus", { focusId });
    },
    appendTask(focusId, text) {
      return runInvoke("append_task", { focusId, text });
    },
    deleteTask(focusId, index) {
      return runInvoke("delete_task", { focusId, index });
    },
    renameFocus(focusId, title) {
      return runInvoke("rename_focus", { focusId, title });
    },
    updateTask(focusId, index, text) {
      return runInvoke("update_task", { focusId, index, text });
    },
    toggleTask(focusId, index, done) {
      return runInvoke("toggle_task", { focusId, index, done });
    },
    startTimer(owner, preset) {
      return runInvoke("start_timer", { owner, preset });
    },
    clearTimer(owner) {
      return runInvoke("clear_timer", { owner });
    },
  };
}
