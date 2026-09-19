import { useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import type { PolledReader } from "../api/polledReader";
import { useAnimalSelection } from "../hooks/useAnimalSelection";
import { useAnimals } from "../hooks/useAnimals";
import { useConfirmDelete } from "../hooks/useConfirmDelete";
import { useDebugOverlay } from "../hooks/useDebugOverlay";
import { type ReportWriteFailure, useFocusController } from "../hooks/useFocusController";
import { PIG_SIZE, usePigMovement } from "../hooks/usePigMovement";
import { usePolledReader } from "../hooks/usePolledReader";
import { useViewport } from "../hooks/useViewport";
import type { AgentSession } from "../types/agentSession";
import type { Focus } from "../types/focus";
import type { TimerPreset } from "../types/timer";
import { AnimalDetail } from "./AnimalDetail";
import { PenBox } from "./PenBox";
import { PigSprite } from "./PigSprite";

export interface AppProps {
  readonly focusReader: PolledReader<readonly Focus[]>;
  readonly focusWriter: FocusWriter;
  readonly onWriteFailure: ReportWriteFailure;
  readonly agentSessionReader: PolledReader<readonly AgentSession[]>;
}

const EMPTY_FOCUSES: readonly Focus[] = [];
const EMPTY_SESSIONS: readonly AgentSession[] = [];

export function App({ focusReader, focusWriter, onWriteFailure, agentSessionReader }: AppProps) {
  const focusController = useFocusController(focusWriter, onWriteFailure);
  const focusState = usePolledReader(focusReader);
  const readerFocuses = focusState.status === "ready" ? focusState.value : EMPTY_FOCUSES;
  const [optimisticFocuses, setOptimisticFocuses] = useState<{
    readonly source: readonly Focus[];
    readonly value: readonly Focus[];
  } | null>(null);
  const focuses =
    optimisticFocuses?.source === readerFocuses ? optimisticFocuses.value : readerFocuses;
  const sessionState = usePolledReader(agentSessionReader);
  const sessions = sessionState.status === "ready" ? sessionState.value : EMPTY_SESSIONS;
  const animals = useAnimals(focuses, sessions);
  const animalsById = new Map(animals.map((animal) => [animal.id, animal]));
  const { selected, select, close } = useAnimalSelection(animals);
  const selectedFocus = selected?.focus ?? null;
  const confirmDelete = useConfirmDelete();
  const { pigs, pens, startDrag, moveDrag, endDrag, setDragActive } = usePigMovement(
    animals,
    selected?.id ?? null,
  );
  const { screenW, screenH } = useViewport();
  const { visible: showDebug, topOffset: debugTopOffset } = useDebugOverlay();

  const selectedPig = pigs.find((p) => p.id === selected?.id);

  async function handleClearTask(index: number) {
    if (!selectedFocus) return;
    await focusController.deleteTask(selectedFocus.id, index);
  }

  async function handleAddTask(text: string) {
    if (!selectedFocus) return;
    const focusId = selectedFocus.id;
    const outcome = await focusController.appendTask(focusId, text);
    if (!outcome.ok) return;
    const newTask = {
      id: `optimistic-${focusId}-${selectedFocus.tasks.length}-${Date.now()}`,
      text,
      done: false,
      timer: null,
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
    await focusController.renameFocus(focusId, title);
  }

  async function handleDuplicateFocus(focusId: string) {
    await focusController.duplicateFocus(focusId);
  }

  async function handleUpdateTask(focusId: string, index: number, text: string) {
    await focusController.updateTask(focusId, index, text);
  }

  async function handleToggleTask(focusId: string, index: number, done: boolean) {
    await focusController.toggleTask(focusId, index, done);
  }

  async function handleDeleteFocus(focusId: string) {
    await focusController.deleteFocus(focusId);
  }

  async function handleStartTimer(focusId: string, preset: TimerPreset) {
    await focusController.startTimer(focusId, preset);
  }

  async function handleClearTimer(focusId: string) {
    await focusController.clearTimer(focusId);
  }

  async function handleStartTaskTimer(focusId: string, index: number, preset: TimerPreset) {
    await focusController.startTaskTimer(focusId, index, preset);
  }

  async function handleClearTaskTimer(focusId: string, index: number) {
    await focusController.clearTaskTimer(focusId, index);
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
          overlay-debug | w={screenW} h={screenH} | focuses={focuses.length} pigs={pigs.length}{" "}
          pens={pens.length}
        </div>
      )}
      {pens.map((layout) => (
        <PenBox key={layout.pen.id} layout={layout} />
      ))}
      {pigs.map((pig) => {
        const animal = animalsById.get(pig.id);
        if (!animal) return null;
        return (
          <PigSprite
            key={pig.id}
            x={pig.x}
            y={pig.y}
            direction={pig.direction}
            frame={pig.frameIndex}
            name={pig.name}
            scale={animal.scale}
            resting={animal.resting}
            onClick={() => select(pig.id)}
            onDragStart={(x, y) => startDrag(pig.id, x, y)}
            onDragMove={moveDrag}
            onDragEnd={endDrag}
            onSetDragActive={setDragActive}
          />
        );
      })}
      {selectedPig && selected && selectedFocus && (
        <AnimalDetail
          focus={selectedFocus}
          animalX={selectedPig.x}
          animalY={selectedPig.y}
          animalSize={PIG_SIZE * selected.scale}
          viewportW={screenW}
          viewportH={screenH}
          confirmDelete={confirmDelete}
          onClose={close}
          onClearTask={handleClearTask}
          onAddTask={handleAddTask}
          onRenameFocus={handleRenameFocus}
          onUpdateTask={handleUpdateTask}
          onToggleTask={handleToggleTask}
          onDuplicateFocus={handleDuplicateFocus}
          onDeleteFocus={handleDeleteFocus}
          onStartTimer={handleStartTimer}
          onClearTimer={handleClearTimer}
          onStartTaskTimer={handleStartTaskTimer}
          onClearTaskTimer={handleClearTaskTimer}
        />
      )}
    </div>
  );
}
