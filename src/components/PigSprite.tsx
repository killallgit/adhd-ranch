export interface PigSpriteProps {
  readonly x: number;
  readonly y: number;
  readonly direction: "left" | "right";
  readonly frame: number;
  readonly name: string;
  readonly onClick: () => void;
}

// Placeholder pig: emoji + name label.
// When real sprite sheet is ready, replace the inner div with:
//   <div
//     className="pig-sprite-frame"
//     style={{
//       backgroundImage: `url(/assets/pig-spritesheet.png)`,
//       backgroundPosition: `-${frame * FRAME_W}px -${row * FRAME_H}px`,
//       width: FRAME_W,
//       height: FRAME_H,
//       imageRendering: "pixelated",
//     }}
//   />
// where row = direction === "left" ? 1 : 2 (adjust to match your sheet layout).
const BOB_OFFSETS = [0, -2, 0, -1];

export function PigSprite({ x, y, direction, frame, name, onClick }: PigSpriteProps) {
  const bob = BOB_OFFSETS[frame % BOB_OFFSETS.length];
  return (
    <button
      type="button"
      className="pig-sprite"
      style={{
        left: x,
        top: y + bob,
        transform: direction === "left" ? "scaleX(-1)" : undefined,
      }}
      onClick={onClick}
    >
      <span className="pig-emoji" aria-hidden="true">
        🐷
      </span>
      <span
        className="pig-name"
        style={{ transform: direction === "left" ? "scaleX(-1)" : undefined }}
      >
        {name}
      </span>
    </button>
  );
}
