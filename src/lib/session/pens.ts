import type { Rect } from "../../types/display";
import type { Pen } from "../../types/generated/Pen";

// The ranch is carved into an even grid, one cell per pen, so a pen's size follows
// only from how many there are — never from how busy it is.
const PEN_GAP = 16;
const PEN_HUES = [12, 45, 96, 145, 195, 250, 290, 330];

// What the settings window offers. The domain clamps a hand-edited file to the same
// range (`MIN_PEN_SIZE` / `MAX_PEN_SIZE` in crates/domain/src/settings.rs), so a value
// from outside it is corrected rather than drawn.
export const PEN_SIZE_RANGE = { min: 160, max: 960 } as const;

export interface PenLayout {
  readonly pen: Pen;
  readonly rect: Rect;
  readonly hue: number;
}

function penHue(penId: string): number {
  let hash = 0;
  for (let i = 0; i < penId.length; i += 1) {
    hash = (hash * 31 + penId.charCodeAt(i)) % 0xffffffff;
  }
  return PEN_HUES[hash % PEN_HUES.length];
}

// Sorted by id so a pen keeps its cell as sessions come and go; an unsorted roster
// would shuffle every animal on screen whenever one session started.
export function layoutPens(
  pens: readonly Pen[],
  area: Rect,
  maxSize: number,
): readonly PenLayout[] {
  if (pens.length === 0) return [];

  const ordered = [...pens].sort((a, b) => a.id.localeCompare(b.id));
  const cols = Math.ceil(Math.sqrt(ordered.length));
  const rows = Math.ceil(ordered.length / cols);
  const penW = Math.max(0, Math.min(area.w / cols - PEN_GAP, maxSize));
  const penH = Math.max(0, Math.min(area.h / rows - PEN_GAP, maxSize));
  const cellW = penW + PEN_GAP;
  const cellH = penH + PEN_GAP;
  // Centred rather than anchored: once the cap bites, the leftover room is split
  // evenly instead of piling up on two sides of the screen.
  const originX = area.x + (area.w - cellW * cols) / 2;
  const originY = area.y + (area.h - cellH * rows) / 2;

  return ordered.map((pen, index) => ({
    pen,
    hue: penHue(pen.id),
    rect: {
      x: originX + (index % cols) * cellW + PEN_GAP / 2,
      y: originY + Math.floor(index / cols) * cellH + PEN_GAP / 2,
      w: penW,
      h: penH,
    },
  }));
}

export function uniquePens(pens: readonly (Pen | null)[]): readonly Pen[] {
  const byId = new Map<string, Pen>();
  for (const pen of pens) {
    if (pen) byId.set(pen.id, pen);
  }
  return [...byId.values()];
}
