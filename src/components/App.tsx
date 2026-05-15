import { useEffect, useState } from "react";
import type { FocusWriter, WriteOutcome } from "../api/focusWriter";
import { subscribeOpenFocusDetail } from "../api/pig";
import type { PolledReader } from "../api/polledReader";
import { useConfirmDelete } from "../hooks/useConfirmDelete";
import { useDebugOverlay } from "../hooks/useDebugOverlay";
import { PIG_SIZE, usePigMovement } from "../hooks/usePigMovement";
import { usePolledReader } from "../hooks/usePolledReader";
import { ranchAnimalScale } from "../hooks/useRanchAnimalScale";
import { useViewport } from "../hooks/useViewport";
import type { Focus } from "../types/focus";
import type { TimerPreset } from "../types/timer";
import { PigDetail } from "./PigDetail";
import { PigSprite } from "./PigSprite";

export type ReportWriteFailure = (op: string, outcome: WriteOutcome) => void;

export interface AppProps {
  readonly focusReader: PolledReader<readonly Focus[]>;
  readonly focusWriter: FocusWriter;
  readonly onWriteFailure: ReportWriteFailure;
}

const EMPTY_FOCUSES: readonly Focus[] = [];

export function App({ focusReader, focusWriter, onWriteFailure }: AppProps) {
  const focusState = usePolledReader(focusReader);
  const readerFocuses = focusState.status === "ready" ? focusState.value : EMPTY_FOCUSES;
  const [optimisticFocuses, setOptimisticFocuses] = useState<{
    readonly source: readonly Focus[];
    readonly value: readonly Focus[];
  } | null>(null);
  const focuses =
    optimisticFocuses?.source === readerFocuses ? optimisticFocuses.value : readerFocuses;
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const confirmDelete = useConfirmDelete();
  const animalScales = new Map(
    focuses.map((focus) => [
      focus.id,
      ranchAnimalScale(focus.timer?.started_at ?? null, focus.timer?.duration_secs ?? null),
    ]),
  );
  const { pigs, startDrag, moveDrag, endDrag, setDragActive } = usePigMovement(
    focuses,
    selectedId,
    animalScales,
  );
  const { screenW, screenH } = useViewport();
  const { visible: showDebug, topOffset: debugTopOffset } = useDebugOverlay();

  const selectedPig = pigs.find((p) => p.id === selectedId);
  const selectedFocus = focuses.find((f) => f.id === selectedId);

  useEffect(() => {
    const unsubscribe = subscribeOpenFocusDetail((focusId) => {
      if (focuses.some((focus) => focus.id === focusId)) {
        setSelectedId(focusId);
      }
    }).catch(() => () => {});
    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, [focuses]);

  async function handleClearTask(index: number) {
    if (!selectedFocus) return;
    onWriteFailure("delete_task", await focusWriter.deleteTask(selectedFocus.id, index));
  }

  async function handleAddTask(text: string) {
    if (!selectedFocus) return;
    const focusId = selectedFocus.id;
    const outcome = await focusWriter.appendTask(focusId, text);
    onWriteFailure("append_task", outcome);
    if (!outcome.ok) return;
    const newTask = {
      id: `optimistic-${focusId}-${selectedFocus.tasks.length}-${Date.now()}`,
      text,
      done: false,
    };
    setOptimisticFocuses({
      source: readerFocuses,
      value: focuses.map((focus) =>
        focus.id === focusId
          ? {
              ...focus,
              tasks: [...focus.tasks, newTask],
              timer: focus.timer?.status === "Expired" ? null : focus.timer,
            }
          : focus,
      ),
    });
  }

  async function handleRenameFocus(focusId: string, title: string) {
    onWriteFailure("rename_focus", await focusWriter.renameFocus(focusId, title));
  }

  async function handleUpdateTask(focusId: string, index: number, text: string) {
    onWriteFailure("update_task", await focusWriter.updateTask(focusId, index, text));
  }

  async function handleToggleTask(focusId: string, index: number, done: boolean) {
    onWriteFailure("toggle_task", await focusWriter.toggleTask(focusId, index, done));
  }

  async function handleDeleteFocus(focusId: string) {
    onWriteFailure("delete_focus", await focusWriter.deleteFocus(focusId));
  }

  async function handleStartTimer(focusId: string, preset: TimerPreset) {
    onWriteFailure("start_timer", await focusWriter.startTimer(focusId, preset));
  }

  return (
    <div className="overlay-root">
      {showDebug && (
        <div
          style={{
            position: "fixed",
            top: debugTopOffset,
            left: 0,
            right: 0,
            background: "rgba(220,0,0,0.85)",
            color: "#fff",
            fontSize: 11,
            padding: "3px 8px",
            zIndex: 9999,
            pointerEvents: "none",
            fontFamily: "monospace",
          }}
        >
          overlay-debug | w={screenW} h={screenH} | focuses={focuses.length} pigs={pigs.length}
        </div>
      )}
      {pigs.map((pig) => (
        <PigSprite
          key={pig.id}
          x={pig.x}
          y={pig.y}
          direction={pig.direction}
          frame={pig.frameIndex}
          name={pig.name}
          scale={animalScales.get(pig.id) ?? 1}
          expired={focuses.find((f) => f.id === pig.id)?.timer?.status === "Expired"}
          onClick={() => setSelectedId(pig.id)}
          onDragStart={(x, y) => startDrag(pig.id, x, y)}
          onDragMove={moveDrag}
          onDragEnd={endDrag}
          onSetDragActive={setDragActive}
        />
      ))}
      {selectedPig && selectedFocus && (
        <PigDetail
          focus={selectedFocus}
          pigX={selectedPig.x}
          pigY={selectedPig.y}
          animalSize={PIG_SIZE * (animalScales.get(selectedPig.id) ?? 1)}
          viewportW={screenW}
          viewportH={screenH}
          confirmDelete={confirmDelete}
          onClose={() => setSelectedId(null)}
          onClearTask={handleClearTask}
          onAddTask={handleAddTask}
          onRenameFocus={handleRenameFocus}
          onUpdateTask={handleUpdateTask}
          onToggleTask={handleToggleTask}
          onDeleteFocus={handleDeleteFocus}
          onStartTimer={handleStartTimer}
        />
      )}
    </div>
  );
}
