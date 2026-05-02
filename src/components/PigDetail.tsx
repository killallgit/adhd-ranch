import { useEffect } from "react";
import { PIG_SIZE } from "../hooks/usePigMovement";
import type { Focus } from "../types/focus";

export interface PigDetailProps {
  readonly focus: Focus;
  readonly pigX: number;
  readonly pigY: number;
  readonly viewportW: number;
  readonly viewportH: number;
  readonly onClose: () => void;
  readonly onClearTask: (index: number) => void;
}

const CARD_W = 210;
const CARD_OFFSET_X = PIG_SIZE + 8;

export function PigDetail({
  focus,
  pigX,
  pigY,
  viewportW,
  viewportH,
  onClose,
  onClearTask,
}: PigDetailProps) {
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
        <h3 className="pig-detail-title">{focus.title}</h3>
        {focus.tasks.length === 0 ? (
          <p className="pig-detail-empty">No tasks yet.</p>
        ) : (
          <ul className="pig-detail-tasks">
            {focus.tasks.map((task, index) => (
              <li key={task.id} className="pig-detail-task">
                <span className="pig-detail-task-text">{task.text}</span>
                <button
                  type="button"
                  className="pig-detail-task-clear"
                  onClick={() => onClearTask(index)}
                  aria-label={`clear task: ${task.text}`}
                >
                  ✗
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </>
  );
}
