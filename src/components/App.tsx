import { useState } from "react";
import type { FocusWriter } from "../api/focusWriter";
import type { FocusReader } from "../api/focuses";
import { useFocuses } from "../hooks/useFocuses";
import { usePigMovement } from "../hooks/usePigMovement";
import { PigDetail } from "./PigDetail";
import { PigSprite } from "./PigSprite";

export interface AppProps {
  readonly focusReader: FocusReader;
  readonly focusWriter: FocusWriter;
}

export function App({ focusReader, focusWriter }: AppProps) {
  const focusState = useFocuses(focusReader);
  const focuses = focusState.status === "ready" ? focusState.focuses : [];
  const pigs = usePigMovement(focuses);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const selectedPig = pigs.find((p) => p.id === selectedId);
  const selectedFocus = focuses.find((f) => f.id === selectedId);

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
          onClose={() => setSelectedId(null)}
          onClearTask={(index) => {
            void focusWriter.deleteTask(selectedFocus.id, index);
          }}
        />
      )}
    </div>
  );
}
