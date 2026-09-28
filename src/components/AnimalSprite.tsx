import type { AnimalMovementState } from "../lib/animalMovement";
import type { Animal } from "../types/generated/Animal";
import { PigSprite } from "./PigSprite";

interface AnimalSpriteProps {
  readonly animal: Animal;
  readonly movement: AnimalMovementState;
  readonly size: number;
  readonly onClick: () => void;
  readonly onDragStart: (x: number, y: number) => void;
  readonly onDragMove: (x: number, y: number) => void;
  readonly onDragEnd: () => { wasDrag: boolean };
  readonly onSetDragActive: (active: boolean) => void;
}

export function AnimalSprite({ animal, movement, size, ...interaction }: AnimalSpriteProps) {
  switch (animal.species) {
    case "pig":
      return (
        <PigSprite
          x={movement.x}
          y={movement.y}
          direction={movement.direction}
          frame={movement.frameIndex}
          label={animal.label}
          size={size}
          motion={animal.motion}
          {...interaction}
        />
      );
    default:
      return assertNever(animal.species);
  }
}

function assertNever(value: never): never {
  throw new Error(`unsupported Animal Species: ${String(value)}`);
}
