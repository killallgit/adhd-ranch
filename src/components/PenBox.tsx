import type { PenLayout } from "../lib/session/pens";

export interface PenBoxProps {
  readonly layout: PenLayout;
}

// Decorative only: the overlay's click-through regions are the animal hit-rects that
// Rust hit-tests, so a pen must never take the pointer.
export function PenBox({ layout }: PenBoxProps) {
  const { rect, hue, pen } = layout;
  return (
    <div
      className="pen-box"
      style={{
        left: rect.x,
        top: rect.y,
        width: rect.w,
        height: rect.h,
        borderColor: `hsl(${hue}, 85%, 60%)`,
        backgroundColor: `hsla(${hue}, 85%, 60%, 0.07)`,
      }}
    >
      <span className="pen-box-label" style={{ color: `hsl(${hue}, 90%, 72%)` }}>
        {pen.name}
      </span>
    </div>
  );
}
