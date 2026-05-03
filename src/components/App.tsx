import { useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import type { FocusReader } from "../api/focuses";
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
  const pigs = usePigMovement(focuses, selectedId);
  const { screenW, screenH } = useViewport();

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
      {pigs.map((pig) => (
        <PigSprite
          key={pig.id}
          x={pig.x}
          y={pig.y}
          direction={pig.direction}
          frame={pig.frameIndex}
          name={pig.name}
          onClick={() => setSelectedId(pig.id)}
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
