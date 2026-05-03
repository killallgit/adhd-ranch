import pigSheet from "../assets/pig-spritesheet.png";
import type { PigDirection } from "../hooks/usePigMovement";
import { PIG_SIZE } from "../hooks/usePigMovement";

const SHEET_COLS = 4;
const DIRECTION_ROW: Record<PigDirection, number> = {
  front: 0,
  right: 1,
  back: 2,
  left: 3,
};

// Bob offsets per frame (subtle vertical hop while walking).
const BOB_OFFSETS = [0, -2, 0, -1];

export interface PigSpriteProps {
  readonly x: number;
  readonly y: number;
  readonly direction: PigDirection;
  readonly frame: number;
  readonly name: string;
  readonly onClick: () => void;
}

export function PigSprite({ x, y, direction, frame, name, onClick }: PigSpriteProps) {
  const bob = BOB_OFFSETS[frame % BOB_OFFSETS.length];
  const col = frame % SHEET_COLS;
  const row = DIRECTION_ROW[direction];
  const sheetSize = PIG_SIZE * SHEET_COLS;

  return (
    <button
      type="button"
      className="pig-sprite"
      style={{ left: x, top: y + bob }}
      onClick={onClick}
    >
      <div
        className="pig-sprite-frame"
        style={{
          backgroundImage: `url(${pigSheet})`,
          backgroundSize: `${sheetSize}px ${sheetSize}px`,
          backgroundPosition: `-${col * PIG_SIZE}px -${row * PIG_SIZE}px`,
          width: PIG_SIZE,
          height: PIG_SIZE,
        }}
      />
      <span className="pig-name">{name}</span>
    </button>
  );
}
