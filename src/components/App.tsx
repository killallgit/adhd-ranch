import { useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import type { PolledReader } from "../api/polledReader";
import type { RenderSceneReader } from "../api/tauriRenderSceneReader";
import { useAnimalMovement } from "../hooks/useAnimalMovement";
import { useConfirmDelete } from "../hooks/useConfirmDelete";
import { useDebugOverlay } from "../hooks/useDebugOverlay";
import { type ReportWriteFailure, useFocusController } from "../hooks/useFocusController";
import { useFocusSelection } from "../hooks/useFocusSelection";
import { usePolledReader } from "../hooks/usePolledReader";
import { useRenderScene } from "../hooks/useRenderScene";
import { useViewport } from "../hooks/useViewport";
import { sampleAnimalSize } from "../lib/animalSize";
import type { Focus } from "../types/focus";
import type { TimerPreset } from "../types/timer";
import { AnimalDetail } from "./AnimalDetail";
import { AnimalSprite } from "./AnimalSprite";
import { RegionBox } from "./RegionBox";

export interface AppProps {
  readonly focusReader: PolledReader<readonly Focus[]>;
  readonly focusWriter: FocusWriter;
  readonly onWriteFailure: ReportWriteFailure;
  readonly renderSceneReader: RenderSceneReader;
}

const EMPTY_FOCUSES: readonly Focus[] = [];

export function App({ focusReader, focusWriter, onWriteFailure, renderSceneReader }: AppProps) {
  const focusController = useFocusController(focusWriter, onWriteFailure);
  const focusState = usePolledReader(focusReader);
  const readerFocuses = focusState.status === "ready" ? focusState.value : EMPTY_FOCUSES;
  const [optimisticFocuses, setOptimisticFocuses] = useState<{
    readonly source: readonly Focus[];
    readonly value: readonly Focus[];
  } | null>(null);
  const focuses =
    optimisticFocuses?.source === readerFocuses ? optimisticFocuses.value : readerFocuses;
  const { scene, displaySpace } = useRenderScene(renderSceneReader);
  const animalsById = new Map(scene.animals.map((animal) => [animal.id, animal]));
  const { selected: selectedFocus, request, close } = useFocusSelection(focuses);
  const confirmDelete = useConfirmDelete();
  const {
    animals: movement,
    startDrag,
    moveDrag,
    endDrag,
    setDragActive,
  } = useAnimalMovement(scene, displaySpace, selectedFocus?.id ?? null);
  const { screenW, screenH } = useViewport();
  const { visible: showDebug, topOffset: debugTopOffset } = useDebugOverlay();

  const selectedMovement = movement.find((state) => state.id === selectedFocus?.id);
  const selectedAnimal = scene.animals.find((animal) => animal.id === selectedFocus?.id);

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
          overlay-debug | w={screenW} h={screenH} | focuses={focuses.length} animals=
          {movement.length} regions={scene.regions.length}
        </div>
      )}
      {scene.regions.map((layout) => (
        <RegionBox key={layout.id} layout={layout} />
      ))}
      {movement.map((state) => {
        const animal = animalsById.get(state.id);
        if (!animal) return null;
        return (
          <AnimalSprite
            key={state.id}
            animal={animal}
            movement={state}
            size={sampleAnimalSize(animal.size, Date.now())}
            onClick={() => request(state.id)}
            onDragStart={(x, y) => startDrag(state.id, x, y)}
            onDragMove={moveDrag}
            onDragEnd={endDrag}
            onSetDragActive={setDragActive}
          />
        );
      })}
      {selectedMovement && selectedAnimal && selectedFocus && (
        <AnimalDetail
          focus={selectedFocus}
          animalX={selectedMovement.x}
          animalY={selectedMovement.y}
          animalSize={sampleAnimalSize(selectedAnimal.size, Date.now())}
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
