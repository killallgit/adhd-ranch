import { useEffect, useState } from "react";
import { PIG_SIZE } from "../hooks/usePigMovement";
import type { Focus } from "../types/focus";
import type { TimerPreset } from "../types/timer";
import { TimerDropdown } from "./TimerDropdown";

export interface AnimalDetailProps {
  readonly focus: Focus;
  readonly animalX: number;
  readonly animalY: number;
  readonly animalSize?: number;
  readonly viewportW: number;
  readonly viewportH: number;
  readonly confirmDelete: boolean;
  readonly onClose: () => void;
  readonly onClearTask: (index: number) => void;
  readonly onAddTask: (text: string) => void;
  readonly onRenameFocus: (focusId: string, title: string) => void;
  readonly onUpdateTask: (focusId: string, index: number, text: string) => void;
  readonly onToggleTask: (focusId: string, index: number, done: boolean) => void;
  readonly onDeleteFocus: (focusId: string) => void;
  readonly onStartTimer: (focusId: string, preset: TimerPreset) => void;
  readonly onClearTimer: (focusId: string) => void;
  readonly onStartTaskTimer: (focusId: string, index: number, preset: TimerPreset) => void;
  readonly onClearTaskTimer: (focusId: string, index: number) => void;
}

const CARD_W = 340;
export function AnimalDetail({
  focus,
  animalX,
  animalY,
  animalSize = PIG_SIZE,
  viewportW,
  viewportH,
  confirmDelete,
  onClose,
  onClearTask,
  onAddTask,
  onRenameFocus,
  onUpdateTask,
  onToggleTask,
  onDeleteFocus,
  onStartTimer,
  onClearTimer,
  onStartTaskTimer,
  onClearTaskTimer,
}: AnimalDetailProps) {
  const [taskInput, setTaskInput] = useState("");
  const [titleDraft, setTitleDraft] = useState(focus.title);
  const [titleError, setTitleError] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  useEffect(() => {
    setTitleDraft(focus.title);
    setTitleError(false);
  }, [focus.title]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  const rawX = animalX + animalSize + 8;
  const x = Math.min(rawX, viewportW - CARD_W - 16);
  const y = Math.max(16, Math.min(animalY, viewportH - 200));

  function commitTitle(): boolean {
    const trimmed = titleDraft.trim();
    if (trimmed === "") {
      setTitleDraft(focus.title);
      setTitleError(true);
      return false;
    }
    setTitleError(false);
    if (trimmed !== focus.title) {
      onRenameFocus(focus.id, trimmed);
    }
    return true;
  }

  function handleDeleteClick() {
    if (confirmDelete) {
      setConfirmingDelete(true);
    } else {
      onDeleteFocus(focus.id);
      onClose();
    }
  }

  function handleConfirmDelete() {
    setConfirmingDelete(false);
    onDeleteFocus(focus.id);
    onClose();
  }

  return (
    <>
      <div
        className="animal-detail-backdrop"
        onClick={onClose}
        onKeyDown={(e) => {
          if (e.key === "Escape") onClose();
        }}
        role="presentation"
      />
      <div className="animal-detail" style={{ left: x, top: y, width: CARD_W }}>
        {confirmingDelete ? (
          <div
            data-testid="animal-detail-delete-confirm"
            className="animal-detail-delete-confirm"
            role="alert"
          >
            <span>Delete "{focus.title}"?</span>
            <button
              type="button"
              className="animal-detail-delete-confirm-yes"
              onClick={handleConfirmDelete}
            >
              Delete
            </button>
            <button
              type="button"
              className="animal-detail-delete-confirm-no"
              onClick={() => setConfirmingDelete(false)}
            >
              Cancel
            </button>
          </div>
        ) : (
          <header className="animal-detail-header">
            <input
              className="animal-detail-title-input"
              aria-label="focus title"
              value={titleDraft}
              onChange={(e) => {
                setTitleDraft(e.target.value);
                if (titleError) setTitleError(false);
              }}
              onBlur={() => {
                commitTitle();
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  if (commitTitle()) {
                    onClose();
                  }
                } else if (e.key === "Escape") {
                  setTitleDraft(focus.title);
                  setTitleError(false);
                  e.currentTarget.blur();
                }
              }}
            />
            <TimerDropdown
              key={focus.id}
              timer={focus.timer}
              ariaLabel={`edit focus timer: ${focus.title}`}
              onStart={(preset) => onStartTimer(focus.id, preset)}
              onClear={() => onClearTimer(focus.id)}
            />
            <button
              type="button"
              className="animal-detail-delete"
              aria-label={`delete focus ${focus.title}`}
              onClick={handleDeleteClick}
            >
              🗑
            </button>
          </header>
        )}
        {titleError && (
          <p className="animal-detail-title-error" role="alert">
            Title cannot be empty
          </p>
        )}
        {focus.tasks.length === 0 ? (
          <p className="animal-detail-empty">No tasks yet.</p>
        ) : (
          <ul className="animal-detail-tasks">
            {focus.tasks.map((task, index) => (
              <TaskEditor
                key={task.id}
                focusId={focus.id}
                index={index}
                task={task}
                onUpdateTask={onUpdateTask}
                onToggleTask={onToggleTask}
                onClearTask={onClearTask}
                onStartTaskTimer={onStartTaskTimer}
                onClearTaskTimer={onClearTaskTimer}
              />
            ))}
          </ul>
        )}
        <input
          className="animal-detail-add-task"
          placeholder="Add task…"
          value={taskInput}
          onChange={(e) => setTaskInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && taskInput.trim()) {
              onAddTask(taskInput.trim());
              setTaskInput("");
            }
          }}
        />
      </div>
    </>
  );
}

interface TaskEditorProps {
  readonly focusId: string;
  readonly index: number;
  readonly task: Focus["tasks"][number];
  readonly onUpdateTask: (focusId: string, index: number, text: string) => void;
  readonly onToggleTask: (focusId: string, index: number, done: boolean) => void;
  readonly onClearTask: (index: number) => void;
  readonly onStartTaskTimer: (focusId: string, index: number, preset: TimerPreset) => void;
  readonly onClearTaskTimer: (focusId: string, index: number) => void;
}

function TaskEditor({
  focusId,
  index,
  task,
  onUpdateTask,
  onToggleTask,
  onClearTask,
  onStartTaskTimer,
  onClearTaskTimer,
}: TaskEditorProps) {
  const [draft, setDraft] = useState(task.text);

  useEffect(() => {
    setDraft(task.text);
  }, [task.text]);

  function commit() {
    const trimmed = draft.trim();
    if (trimmed === "" || trimmed === task.text) {
      setDraft(task.text);
      return;
    }
    onUpdateTask(focusId, index, trimmed);
  }

  return (
    <li className={`animal-detail-task${task.done ? " animal-detail-task--done" : ""}`}>
      <input
        type="checkbox"
        className="animal-detail-task-check"
        aria-label={`toggle task: ${task.text}`}
        checked={task.done}
        onChange={(e) => onToggleTask(focusId, index, e.target.checked)}
      />
      <input
        className="animal-detail-task-input"
        aria-label={`task text: ${task.text}`}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.currentTarget.blur();
          } else if (e.key === "Escape") {
            setDraft(task.text);
            e.currentTarget.blur();
          }
        }}
      />
      <TimerDropdown
        key={task.id}
        timer={task.timer}
        ariaLabel={`edit task timer: ${task.text}`}
        onStart={(preset) => onStartTaskTimer(focusId, index, preset)}
        onClear={() => onClearTaskTimer(focusId, index)}
      />
      <button
        type="button"
        className="animal-detail-task-clear"
        onClick={() => onClearTask(index)}
        aria-label={`clear task: ${task.text}`}
      >
        ✗
      </button>
    </li>
  );
}
