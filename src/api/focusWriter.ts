import { invoke } from "@tauri-apps/api/core";
import type { CommandError } from "../types/error";
import type { TimerPreset } from "../types/timer";

export interface FocusWriter {
  createFocus(input: {
    title: string;
    description?: string;
    timer_preset?: TimerPreset | null;
  }): Promise<{ id: string }>;
  deleteFocus(focusId: string): Promise<void>;
  appendTask(focusId: string, text: string): Promise<void>;
  deleteTask(focusId: string, index: number): Promise<void>;
}

function logErr(op: string) {
  return (e: unknown): never => {
    console.error(`[adhd-ranch] ${op}`, e as CommandError);
    throw e;
  };
}

export function createTauriFocusWriter(): FocusWriter {
  return {
    createFocus({ title, description, timer_preset }) {
      return invoke<{ id: string }>("create_focus", {
        title,
        description,
        timerPreset: timer_preset ?? null,
      }).catch(logErr("create_focus"));
    },
    deleteFocus(focusId: string) {
      return invoke<void>("delete_focus", { focusId }).catch(logErr("delete_focus"));
    },
    appendTask(focusId: string, text: string) {
      return invoke<void>("append_task", { focusId, text }).catch(logErr("append_task"));
    },
    deleteTask(focusId: string, index: number) {
      return invoke<void>("delete_task", { focusId, index }).catch(logErr("delete_task"));
    },
  };
}
