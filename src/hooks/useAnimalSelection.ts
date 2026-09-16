import { useCallback, useEffect, useState } from "react";
import { subscribeOpenFocusDetail } from "../api/pig";
import type { Animal, FocusAnimal } from "../types/animal";

export interface AnimalSelection {
  readonly selected: FocusAnimal | null;
  readonly select: (animalId: string) => void;
  readonly close: () => void;
}

// The selection is derived, not stored: an Animal that is gone — deleted from the
// tray, or removed on disk — resolves to none in the same render, so the overlay
// never keeps its full-viewport hit rect for a card nobody can see.
export function useAnimalSelection(animals: readonly Animal[]): AnimalSelection {
  const [requestedId, setRequestedId] = useState<string | null>(null);
  const select = useCallback((animalId: string) => setRequestedId(animalId), []);
  const close = useCallback(() => setRequestedId(null), []);

  useEffect(() => {
    const unsubscribe = subscribeOpenFocusDetail(select).catch(() => () => {});
    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, [select]);

  const requested = animals.find((animal) => animal.id === requestedId);
  const selected = requested?.kind === "focus" ? requested : null;

  return { selected, select, close };
}
