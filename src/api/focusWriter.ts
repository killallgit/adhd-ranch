import { invoke } from "@tauri-apps/api/core";

export interface FocusWriter {
  createFocus(input: { title: string; description?: string }): Promise<{ id: string }>;
  deleteFocus(focusId: string): Promise<void>;
  appendTask(focusId: string, text: string): Promise<void>;
  deleteTask(focusId: string, index: number): Promise<void>;
}

export function createTauriFocusWriter(): FocusWriter {
  return {
    createFocus({ title, description }) {
      return invoke<{ id: string }>("create_focus", { title, description });
    },
    deleteFocus(focusId: string) {
      return invoke<void>("delete_focus", { focusId });
    },
    appendTask(focusId: string, text: string) {
      return invoke<void>("append_task", { focusId, text });
    },
    deleteTask(focusId: string, index: number) {
      return invoke<void>("delete_task", { focusId, index });
    },
  };
}
