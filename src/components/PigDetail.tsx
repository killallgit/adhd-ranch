import { useEffect, useState } from "react";
import { PIG_SIZE } from "../hooks/usePigMovement";
import type { Focus } from "../types/focus";

export interface PigDetailProps {
  readonly focus: Focus;
  readonly pigX: number;
  readonly pigY: number;
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
}

const CARD_W = 340;
const CARD_OFFSET_X = PIG_SIZE + 8;

export function PigDetail({
  focus,
  pigX,
  pigY,
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
}: PigDetailProps) {
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

  const rawX = pigX + CARD_OFFSET_X;
  const x = Math.min(rawX, viewportW - CARD_W - 16);
  const y = Math.max(16, Math.min(pigY, viewportH - 200));

  function commitTitle() {
    const trimmed = titleDraft.trim();
    if (trimmed === "") {
      setTitleDraft(focus.title);
      setTitleError(true);
      return;
    }
    setTitleError(false);
    if (trimmed !== focus.title) {
      onRenameFocus(focus.id, trimmed);
    }
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
        className="pig-detail-backdrop"
        onClick={onClose}
        onKeyDown={(e) => {
          if (e.key === "Escape") onClose();
        }}
        role="presentation"
      />
      <div className="pig-detail" style={{ left: x, top: y, width: CARD_W }}>
        {confirmingDelete ? (
          <div
            data-testid="pig-detail-delete-confirm"
            className="pig-detail-delete-confirm"
            role="alert"
          >
            <span>Delete "{focus.title}"?</span>
            <button
              type="button"
              className="pig-detail-delete-confirm-yes"
              onClick={handleConfirmDelete}
            >
              Delete
            </button>
            <button
              type="button"
              className="pig-detail-delete-confirm-no"
              onClick={() => setConfirmingDelete(false)}
            >
              Cancel
            </button>
          </div>
        ) : (
          <header className="pig-detail-header">
            <input
              className="pig-detail-title-input"
              aria-label="focus title"
              value={titleDraft}
              onChange={(e) => {
                setTitleDraft(e.target.value);
                if (titleError) setTitleError(false);
              }}
              onBlur={commitTitle}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.currentTarget.blur();
                } else if (e.key === "Escape") {
                  setTitleDraft(focus.title);
                  setTitleError(false);
                  e.currentTarget.blur();
                }
              }}
            />
            <button
              type="button"
              className="pig-detail-delete"
              aria-label={`delete focus ${focus.title}`}
              onClick={handleDeleteClick}
            >
              🗑
            </button>
          </header>
        )}
        {titleError && (
          <p className="pig-detail-title-error" role="alert">
            Title cannot be empty
          </p>
        )}
        {focus.tasks.length === 0 ? (
          <p className="pig-detail-empty">No tasks yet.</p>
        ) : (
          <ul className="pig-detail-tasks">
            {focus.tasks.map((task, index) => (
              <TaskEditor
                key={task.id}
                focusId={focus.id}
                index={index}
                task={task}
                onUpdateTask={onUpdateTask}
                onToggleTask={onToggleTask}
                onClearTask={onClearTask}
              />
            ))}
          </ul>
        )}
        <input
          className="pig-detail-add-task"
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
}

function TaskEditor({
  focusId,
  index,
  task,
  onUpdateTask,
  onToggleTask,
  onClearTask,
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
    <li className={`pig-detail-task${task.done ? " pig-detail-task--done" : ""}`}>
      <input
        type="checkbox"
        className="pig-detail-task-check"
        aria-label={`toggle task: ${task.text}`}
        checked={task.done}
        onChange={(e) => onToggleTask(focusId, index, e.target.checked)}
      />
      <input
        className="pig-detail-task-input"
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
      <button
        type="button"
        className="pig-detail-task-clear"
        onClick={() => onClearTask(index)}
        aria-label={`clear task: ${task.text}`}
      >
        ✗
      </button>
    </li>
  );
}
