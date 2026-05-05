import { useCallback, useEffect, useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import type { FocusReader } from "../api/focuses";
import { setPigDragActive, subscribeDisplayRegion, subscribeGatherPigs } from "../api/pig";
import { useDebugOverlay } from "../hooks/useDebugOverlay";
import { useFocuses } from "../hooks/useFocuses";
import { usePigMovement } from "../hooks/usePigMovement";
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
  const { visible: showDebug, topOffset: debugTopOffset } = useDebugOverlay();

  const handleSetDragActive = useCallback((active: boolean) => {
    setPigDragActive(active).catch(() => {});
  }, []);

  useEffect(() => {
    // If subscribe rejects, fall back to a no-op unsubscribe so cleanup never throws
    // an unhandled rejection during React strict-mode unmounts.
    const unsubPromise = subscribeGatherPigs(gather).catch(() => () => {});
    return () => {
      unsubPromise.then((unsub) => unsub());
    };
  }, [gather]);

  useEffect(() => {
    const unsubPromise = subscribeDisplayRegion(setRegion).catch(() => () => {});
    return () => {
      unsubPromise.then((unsub) => unsub());
    };
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
          onSetDragActive={handleSetDragActive}
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
