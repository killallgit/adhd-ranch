import { useEffect, useState } from "react";
import { PIG_SIZE } from "../hooks/usePigMovement";
import { type PresetSelection, isCustomValid, resolvePreset } from "../lib/timerPreset";
import type { Focus } from "../types/focus";
import type { TimerPreset } from "../types/timer";
import { TimerPresetPicker } from "./TimerPresetPicker";

export interface PigDetailProps {
  readonly focus: Focus;
  readonly pigX: number;
  readonly pigY: number;
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
}

const CARD_W = 340;
export function PigDetail({
  focus,
  pigX,
  pigY,
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
}: PigDetailProps) {
  const [taskInput, setTaskInput] = useState("");
  const [titleDraft, setTitleDraft] = useState(focus.title);
  const [titleError, setTitleError] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [timerSelection, setTimerSelection] = useState<PresetSelection>("Eight");
  const [customMinutes, setCustomMinutes] = useState(10);
  const [timerError, setTimerError] = useState<string | null>(null);

  function handleStartTimer() {
    if (timerSelection === "custom" && !isCustomValid(customMinutes)) {
      setTimerError("custom timer must be at least 1 minute");
      return;
    }
    const preset = resolvePreset(timerSelection, customMinutes);
    if (preset === null) return;
    setTimerError(null);
    onStartTimer(focus.id, preset);
    onClose();
  }

  useEffect(() => {
    setTitleDraft(focus.title);
    setTitleError(false);
  }, [focus.title]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: re-run when switching to a different focus so picker state doesn't leak.
  useEffect(() => {
    setTimerSelection("Eight");
    setCustomMinutes(10);
    setTimerError(null);
  }, [focus.id]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  const rawX = pigX + animalSize + 8;
  const x = Math.min(rawX, viewportW - CARD_W - 16);
  const y = Math.max(16, Math.min(pigY, viewportH - 200));

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

  const timerStatus = describeTimerStatus(focus.timer ?? null);

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
        <div className="pig-detail-timer">
          {timerStatus && <span className="pig-detail-timer-status">{timerStatus}</span>}
          <TimerPresetPicker
            selection={timerSelection}
            customMinutes={customMinutes}
            allowNone={false}
            onSelectionChange={(v) => {
              setTimerSelection(v);
              if (timerError) setTimerError(null);
            }}
            onCustomMinutesChange={(v) => {
              setCustomMinutes(v);
              if (timerError) setTimerError(null);
            }}
          />
          <button type="button" className="pig-detail-timer-start" onClick={handleStartTimer}>
            {focus.timer && focus.timer.status === "Running" ? "Restart" : "Start"}
          </button>
          {timerError && (
            <p className="pig-detail-timer-error" role="alert">
              {timerError}
            </p>
          )}
        </div>
      </div>
    </>
  );
}

function describeTimerStatus(timer: Focus["timer"] | null): string | null {
  if (!timer) return null;
  if (timer.status === "Expired") return "Expired";
  const elapsedSecs = Math.max(0, Math.floor(Date.now() / 1000) - timer.started_at);
  const remainingSecs = Math.max(0, timer.duration_secs - elapsedSecs);
  const minutes = Math.floor(remainingSecs / 60)
    .toString()
    .padStart(2, "0");
  const seconds = (remainingSecs % 60).toString().padStart(2, "0");
  return `${minutes}:${seconds} remaining`;
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
