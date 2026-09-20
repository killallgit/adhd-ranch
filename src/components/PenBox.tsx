import type { CSSProperties } from "react";
import type { PenLayout } from "../lib/session/pens";

export interface PenBoxProps {
  readonly layout: PenLayout;
}

// Decorative only: the overlay's click-through regions are the animal hit-rects that
// Rust hit-tests, so a pen must never take the pointer.
export function PenBox({ layout }: PenBoxProps) {
  const { rect, hue, pen } = layout;
  // The fence and the label are both drawn from this one number, so the stylesheet
  // stays the only place that decides what a pen looks like.
  const style = {
    left: rect.x,
    top: rect.y,
    width: rect.w,
    height: rect.h,
    "--pen-hue": String(hue),
  } as CSSProperties;

  return (
    <div className="pen-box" style={style}>
      <span className="pen-box-label">{pen.name}</span>
    </div>
  );
}
