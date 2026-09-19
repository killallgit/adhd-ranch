import type { Rect } from "../../types/display";
import type { Pen } from "../../types/generated/Pen";

// The ranch is carved into an even grid, one cell per pen, so a pen's size follows
// only from how many there are — never from how busy it is.
const PEN_GAP = 16;
const PEN_HUES = [12, 45, 96, 145, 195, 250, 290, 330];

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
export function layoutPens(pens: readonly Pen[], area: Rect): readonly PenLayout[] {
  if (pens.length === 0) return [];

  const ordered = [...pens].sort((a, b) => a.id.localeCompare(b.id));
  const cols = Math.ceil(Math.sqrt(ordered.length));
  const rows = Math.ceil(ordered.length / cols);
  const cellW = area.w / cols;
  const cellH = area.h / rows;

  return ordered.map((pen, index) => ({
    pen,
    hue: penHue(pen.id),
    rect: {
      x: area.x + (index % cols) * cellW + PEN_GAP / 2,
      y: area.y + Math.floor(index / cols) * cellH + PEN_GAP / 2,
      w: Math.max(0, cellW - PEN_GAP),
      h: Math.max(0, cellH - PEN_GAP),
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
