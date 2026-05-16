import { useRef } from "react";
import pigSheet from "../assets/pig-spritesheet.png";
import { DRAG_THRESHOLD, PIG_SIZE } from "../hooks/usePigMovement";
import type { PigDirection } from "../hooks/usePigMovement";

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
  readonly scale?: number;
  readonly expired?: boolean;
  readonly onClick: () => void;
  readonly onDragStart: (x: number, y: number) => void;
  readonly onDragMove: (x: number, y: number) => void;
  readonly onDragEnd: () => { wasDrag: boolean };
  readonly onSetDragActive: (active: boolean) => void;
}

export function PigSprite({
  x,
  y,
  direction,
  frame,
  name,
  scale = 1,
  expired = false,
  onClick,
  onDragStart,
  onDragMove,
  onDragEnd,
  onSetDragActive,
}: PigSpriteProps) {
  const bob = BOB_OFFSETS[frame % BOB_OFFSETS.length];
  const col = frame % SHEET_COLS;
  const row = DIRECTION_ROW[direction];
  const size = PIG_SIZE * scale;
  const sheetSize = size * SHEET_COLS;

  const startPosRef = useRef<{ x: number; y: number } | null>(null);
  const isDraggingRef = useRef(false);

  return (
    <button
      type="button"
      className={`pig-sprite${expired ? " pig-sprite--expired" : ""}`}
      style={{
        left: x,
        top: y + bob,
        cursor: isDraggingRef.current ? "grabbing" : "pointer",
      }}
      onPointerDown={(e) => {
        onSetDragActive(true);
        e.currentTarget.setPointerCapture?.(e.pointerId);
        startPosRef.current = { x: e.clientX, y: e.clientY };
        isDraggingRef.current = false;
        onDragStart(e.clientX, e.clientY);
      }}
      onPointerMove={(e) => {
        if (!startPosRef.current) return;
        const dx = e.clientX - startPosRef.current.x;
        const dy = e.clientY - startPosRef.current.y;
        if (!isDraggingRef.current && Math.sqrt(dx * dx + dy * dy) >= DRAG_THRESHOLD) {
          isDraggingRef.current = true;
        }
        if (isDraggingRef.current) {
          onDragMove(e.clientX, e.clientY);
        }
      }}
      onPointerUp={(e) => {
        onSetDragActive(false);
        e.currentTarget.releasePointerCapture?.(e.pointerId);
        startPosRef.current = null;
        const { wasDrag } = onDragEnd();
        isDraggingRef.current = false;
        if (!wasDrag) onClick();
      }}
      onPointerCancel={(e) => {
        onSetDragActive(false);
        e.currentTarget.releasePointerCapture?.(e.pointerId);
        startPosRef.current = null;
        onDragEnd();
        isDraggingRef.current = false;
      }}
      onLostPointerCapture={() => {
        if (startPosRef.current) {
          onSetDragActive(false);
          startPosRef.current = null;
          onDragEnd();
          isDraggingRef.current = false;
        }
      }}
    >
      <div
        className="pig-sprite-frame"
        style={{
          backgroundImage: `url(${pigSheet})`,
          backgroundSize: `${sheetSize}px ${sheetSize}px`,
          backgroundPosition: `-${col * size}px -${row * size}px`,
          width: size,
          height: size,
          filter: expired
            ? "grayscale(1) saturate(0.15) brightness(1.55) drop-shadow(0 0 8px rgba(210, 240, 255, 0.55))"
            : undefined,
          opacity: expired ? 0.48 : undefined,
        }}
      />
      <span className="pig-name">{name}</span>
    </button>
  );
}
