import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import type { FocusReader } from "../api/focuses";
import { useFocuses } from "../hooks/useFocuses";
import { type SpawnRegion, usePigMovement } from "../hooks/usePigMovement";
import { useViewport } from "../hooks/useViewport";
import { PigDetail } from "./PigDetail";
import { PigSprite } from "./PigSprite";

export interface AppProps {
  readonly focusReader: FocusReader;
  readonly focusWriter: FocusWriter;
}

export function App({ focusReader, focusWriter }: AppProps) {
  const focusState = useFocuses(focusReader);
  const focuses = focusState.status === "ready" ? focusState.focuses : [];
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const { pigs, startDrag, moveDrag, endDrag, gather, setRegion } = usePigMovement(
    focuses,
    selectedId,
  );
  const { screenW, screenH } = useViewport();

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    listen("gather-pigs", gather).then((fn) => {
      cleanup = fn;
    });
    return () => cleanup?.();
  }, [gather]);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    listen<SpawnRegion>("display-region", (event) => {
      setRegion(event.payload);
    }).then((fn) => {
      cleanup = fn;
    });
    return () => cleanup?.();
  }, [setRegion]);

  const selectedPig = pigs.find((p) => p.id === selectedId);
  const selectedFocus = focuses.find((f) => f.id === selectedId);

  async function handleClearTask(index: number) {
    if (!selectedFocus) return;
    try {
      await focusWriter.deleteTask(selectedFocus.id, index);
    } catch {
      // focusWriter already logs the typed error
    }
  }

  async function handleAddTask(text: string) {
    if (!selectedFocus) return;
    try {
      await focusWriter.appendTask(selectedFocus.id, text);
    } catch {
      // focusWriter already logs the typed error
    }
  }

  return (
    <div className="overlay-root">
      {import.meta.env.DEV && (
        <div
          style={{
            position: "fixed",
            top: 0,
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
        />
      ))}
      {selectedPig && selectedFocus && (
        <PigDetail
          focus={selectedFocus}
          pigX={selectedPig.x}
          pigY={selectedPig.y}
          viewportW={screenW}
          viewportH={screenH}
          onClose={() => setSelectedId(null)}
          onClearTask={handleClearTask}
          onAddTask={handleAddTask}
        />
      )}
    </div>
  );
}
