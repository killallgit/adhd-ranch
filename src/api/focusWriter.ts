import { invoke } from "@tauri-apps/api/core";
import type { TimerPreset } from "../types/timer";

export type WriteOutcome =
  | { ok: true }
  | { ok: false; kind: "ipc" | "domain" | "not_found"; message: string };

export interface CreateFocusInput {
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
  startTimer(focusId: string, preset: TimerPreset): Promise<WriteOutcome>;
  clearTimer(focusId: string): Promise<WriteOutcome>;
  startTaskTimer(focusId: string, index: number, preset: TimerPreset): Promise<WriteOutcome>;
  clearTaskTimer(focusId: string, index: number): Promise<WriteOutcome>;
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
    startTimer(focusId, preset) {
      return runInvoke("start_timer", { focusId, preset });
    },
    clearTimer(focusId) {
      return runInvoke("clear_timer", { focusId });
    },
    startTaskTimer(focusId, index, preset) {
      return runInvoke("start_task_timer", { focusId, index, preset });
    },
    clearTaskTimer(focusId, index) {
      return runInvoke("clear_task_timer", { focusId, index });
    },
  };
}
