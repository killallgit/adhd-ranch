import { useState } from "react";
import type { FocusWriter, WriteOutcome } from "../api/focusWriter";
import type { PolledReader } from "../api/polledReader";
import { useConfirmDelete } from "../hooks/useConfirmDelete";
import { useDebugOverlay } from "../hooks/useDebugOverlay";
import { usePigMovement } from "../hooks/usePigMovement";
import { usePolledReader } from "../hooks/usePolledReader";
import { useViewport } from "../hooks/useViewport";
import type { Focus } from "../types/focus";
import { PigDetail } from "./PigDetail";
import { PigSprite } from "./PigSprite";

export type ReportWriteFailure = (op: string, outcome: WriteOutcome) => void;

export interface AppProps {
  readonly focusReader: PolledReader<readonly Focus[]>;
  readonly focusWriter: FocusWriter;
  readonly onWriteFailure: ReportWriteFailure;
}

export function App({ focusReader, focusWriter, onWriteFailure }: AppProps) {
  const focusState = usePolledReader(focusReader);
  const focuses = focusState.status === "ready" ? focusState.value : [];
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const confirmDelete = useConfirmDelete();
  const { pigs, startDrag, moveDrag, endDrag, setDragActive } = usePigMovement(focuses, selectedId);
  const { screenW, screenH } = useViewport();
  const { visible: showDebug, topOffset: debugTopOffset } = useDebugOverlay();

  const selectedPig = pigs.find((p) => p.id === selectedId);
  const selectedFocus = focuses.find((f) => f.id === selectedId);

  async function handleClearTask(index: number) {
    if (!selectedFocus) return;
    onWriteFailure("delete_task", await focusWriter.deleteTask(selectedFocus.id, index));
  }

  async function handleAddTask(text: string) {
    if (!selectedFocus) return;
    onWriteFailure("append_task", await focusWriter.appendTask(selectedFocus.id, text));
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
        />
      )}
    </div>
  );
}
