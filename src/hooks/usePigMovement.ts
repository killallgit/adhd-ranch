import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import {
  setPigDragActive,
  subscribeDisplaySpace,
  subscribeGatherPigs,
  updatePigRects,
} from "../api/pig";
import { animalPen } from "../lib/animals";
import {
  RANCH_ANIMAL_SIZE,
  RANCH_ANIMAL_SPEED,
  type RanchAnimalDirection,
  type RanchAnimalState,
  advanceRanchAnimal,
  edgeMargin,
  restRanchAnimal,
} from "../lib/ranchAnimalMovement";
import { type PenLayout, layoutPens, uniquePens } from "../lib/session/pens";
import type { Animal } from "../types/animal";
import type { DisplaySpace, Rect } from "../types/display";
import type { Pen } from "../types/generated/Pen";
import type { PigHitRect } from "../types/pig";

export type { DisplaySpace, PigHitRect };

export const PIG_SIZE = RANCH_ANIMAL_SIZE;
export const HITBOX_PADDING = 16;
export const DRAG_THRESHOLD = 4;
export const TOSS_VELOCITY_WINDOW_MS = 80;

export const PIG_SPEED = RANCH_ANIMAL_SPEED; // px/s — fast enough to look alive across large spans

const RECT_UPDATE_EVERY = 4; // rAF frames between pig-rect syncs to Rust

export interface PointerSample {
  x: number;
  y: number;
  t: number;
}

export function computeTossVelocity(
  samples: PointerSample[],
  windowMs: number,
  now: number,
): { vx: number; vy: number } {
  const recent = samples.filter((s) => s.t >= now - windowMs);
  if (recent.length < 2) return { vx: 0, vy: 0 };
  const first = recent[0];
  const last = recent[recent.length - 1];
  if (!first || !last) return { vx: 0, vy: 0 };
  const dt = last.t - first.t;
  if (dt === 0) return { vx: 0, vy: 0 };
  const vx = ((last.x - first.x) / dt) * 1000;
  const vy = ((last.y - first.y) / dt) * 1000;
  const maxV = PIG_SPEED * 6;
  const speed = Math.sqrt(vx * vx + vy * vy);
  if (speed > maxV) {
    return { vx: (vx / speed) * maxV, vy: (vy / speed) * maxV };
  }
  return { vx, vy };
}

export type PigDirection = RanchAnimalDirection;

export type PigState = RanchAnimalState;

export interface PigMovementResult {
  pigs: PigState[];
  pens: readonly PenLayout[];
  startDrag: (pigId: string, x: number, y: number) => void;
  moveDrag: (x: number, y: number) => void;
  endDrag: () => { wasDrag: boolean };
  setDragActive: (active: boolean) => void;
}

function direction4(vx: number, vy: number): PigDirection {
  if (Math.abs(vx) >= Math.abs(vy)) return vx >= 0 ? "right" : "left";
  return vy >= 0 ? "front" : "back";
}

interface RosterEntry {
  readonly id: string;
  readonly name: string;
  readonly resting: boolean;
  readonly pen: Pen | null;
}

// An animal with a pen roams only that pen; one without — a focus — has the whole ranch.
function regionsFor(
  penId: string | null | undefined,
  penRects: ReadonlyMap<string, Rect>,
  displaySpace: DisplaySpace,
): readonly Rect[] {
  const rect = penId ? penRects.get(penId) : undefined;
  return rect ? [rect] : displaySpace.movementRegions;
}

function spawnRegionFor(
  penId: string | null,
  penRects: ReadonlyMap<string, Rect>,
  displaySpace: DisplaySpace,
): Rect {
  return (penId ? penRects.get(penId) : undefined) ?? displaySpace.spawnRegion;
}

function initPig(animal: RosterEntry, region: Rect, now: number): PigState {
  const margin = edgeMargin(region);
  const angle = Math.random() * 2 * Math.PI;
  const vx = Math.cos(angle) * PIG_SPEED;
  const vy = Math.sin(angle) * PIG_SPEED;
  return {
    id: animal.id,
    name: animal.name,
    x: region.x + margin + Math.random() * Math.max(0, region.w - 2 * margin - PIG_SIZE),
    y: region.y + margin + Math.random() * Math.max(0, region.h - 2 * margin - PIG_SIZE),
    vx,
    vy,
    frameIndex: 0,
    direction: direction4(vx, vy),
    lastFrameAt: now,
    nextTurnAt: now + 3000 + Math.random() * 5000,
  };
}

export function buildHitRects(
  pigs: PigState[],
  dpr: number,
  animalScales: ReadonlyMap<string, number> = new Map(),
): PigHitRect[] {
  return pigs.map((p) => ({
    x: (p.x - HITBOX_PADDING / 2) * dpr,
    y: (p.y - HITBOX_PADDING / 2) * dpr,
    size: (PIG_SIZE * (animalScales.get(p.id) ?? 1) + HITBOX_PADDING) * dpr,
  }));
}

function syncRects(
  pigs: PigState[],
  wide: boolean,
  displaySpace: DisplaySpace,
  animalScales: ReadonlyMap<string, number> = new Map(),
): void {
  const scale = displaySpace.hitTestScale;
  // Wide rect (detail open or dragging) keeps overlay interactive across the full viewport.
  const rects = wide
    ? [{ x: 0, y: 0, size: Math.max(displaySpace.span.w, displaySpace.span.h) * scale * 2 }]
    : buildHitRects(pigs, scale, animalScales);
  updatePigRects(rects).catch(() => {});
}

function defaultDisplaySpace(): DisplaySpace {
  const w = document.documentElement.clientWidth || window.screen.width;
  const h = document.documentElement.clientHeight || window.screen.height;
  const region = { x: 0, y: 0, w, h };
  return {
    span: { w, h },
    spawnRegion: region,
    movementRegions: [region],
    hitTestScale: window.devicePixelRatio || 1,
  };
}

export function usePigMovement(
  animals: readonly Animal[],
  selectedId: string | null,
): PigMovementResult {
  const [pigs, setPigs] = useState<PigState[]>([]);
  const pigsRef = useRef<PigState[]>([]);
  const selectedIdRef = useRef<string | null>(selectedId);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(performance.now());
  const frameCountRef = useRef<number>(0);
  const fallbackDisplaySpaceRef = useRef<DisplaySpace | null>(null);
  const [displaySpace, setDisplaySpaceState] = useState<DisplaySpace>(() => {
    const space = defaultDisplaySpace();
    fallbackDisplaySpaceRef.current = space;
    return space;
  });
  // DisplaySpace is updated when Rust emits display-space.
  const displaySpaceRef = useRef<DisplaySpace>(displaySpace);
  const animalScalesRef = useRef<ReadonlyMap<string, number>>(new Map());
  const spawnedFromFallbackRef = useRef<Set<string>>(new Set());

  // Drag state — refs to avoid stale closures in the rAF loop.
  const dragIdRef = useRef<string | null>(null);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const pointerHistoryRef = useRef<PointerSample[]>([]);

  // Keep selectedId ref in sync so the rAF loop sees the latest value without restarting.
  selectedIdRef.current = selectedId;
  animalScalesRef.current = new Map(animals.map((animal) => [animal.id, animal.scale]));

  // Scale changes every frame while a timer runs, so `animals` is a fresh array on
  // every render. Spawning and resting only care about who is on the ranch, so the
  // roster is keyed on that and the rAF loop is left alone in between.
  const rosterKey = animals
    .map(
      (animal) =>
        `${animal.id}\u0000${animal.name}\u0000${animal.resting}\u0000${animalPen(animal)?.id ?? ""}`,
    )
    .join("\u0001");
  // biome-ignore lint/correctness/useExhaustiveDependencies: rosterKey is the value identity of animals
  const roster = useMemo<readonly RosterEntry[]>(
    () =>
      animals.map((animal) => ({
        id: animal.id,
        name: animal.name,
        resting: animal.resting,
        pen: animalPen(animal),
      })),
    [rosterKey],
  );

  const pens = useMemo(
    () => layoutPens(uniquePens(roster.map((entry) => entry.pen)), displaySpace.spawnRegion),
    [roster, displaySpace],
  );
  const penRects = useMemo(
    () => new Map(pens.map((layout) => [layout.pen.id, layout.rect])),
    [pens],
  );
  const penIdByAnimal = useMemo(
    () =>
      new Map(roster.flatMap((entry) => (entry.pen ? [[entry.id, entry.pen.id] as const] : []))),
    [roster],
  );
  const penRectsRef = useRef<ReadonlyMap<string, Rect>>(penRects);
  const penIdByAnimalRef = useRef<ReadonlyMap<string, string>>(penIdByAnimal);
  // Written after commit rather than during render: the animation frame must never
  // be able to read a layout from a render React went on to throw away.
  useLayoutEffect(() => {
    penRectsRef.current = penRects;
    penIdByAnimalRef.current = penIdByAnimal;
  }, [penRects, penIdByAnimal]);

  const setDisplaySpace = useCallback((space: DisplaySpace) => {
    displaySpaceRef.current = space;
    setDisplaySpaceState(space);
  }, []);

  const setDragActive = useCallback((active: boolean) => {
    setPigDragActive(active).catch(() => {});
  }, []);

  const startDrag = useCallback((pigId: string, x: number, y: number) => {
    dragIdRef.current = pigId;
    dragStartRef.current = { x, y };
    pointerHistoryRef.current = [{ x, y, t: performance.now() }];
    // Widen hit-rect immediately so the overlay stays interactive during the drag.
    // Without this there is a ~67ms window where the window is click-through,
    // which breaks pointer capture when crossing monitor boundaries.
    syncRects(pigsRef.current, true, displaySpaceRef.current, animalScalesRef.current);
  }, []);

  const moveDrag = useCallback((x: number, y: number) => {
    if (!dragIdRef.current) return;
    const now = performance.now();
    pointerHistoryRef.current.push({ x, y, t: now });
    // Keep only last 200ms of history to bound memory.
    pointerHistoryRef.current = pointerHistoryRef.current.filter((s) => s.t >= now - 200);

    setPigs((prev) => {
      const next = prev.map((p) =>
        p.id === dragIdRef.current ? { ...p, x, y, direction: direction4(p.vx, p.vy) } : p,
      );
      pigsRef.current = next;
      return next;
    });
  }, []);

  const gather = useCallback(() => {
    setPigs((prev) => {
      const r = displaySpaceRef.current.spawnRegion;
      const margin = 20;
      const rowHeight = PIG_SIZE + 24;
      const colWidth = PIG_SIZE + 24;
      const rows = Math.max(1, Math.floor((r.h - margin * 2) / rowHeight));
      const next = prev.map((p, i) => ({
        ...p,
        x: r.x + r.w - margin - PIG_SIZE - Math.floor(i / rows) * colWidth,
        y: r.y + margin + (i % rows) * rowHeight,
        vx: 0,
        vy: 0,
      }));
      pigsRef.current = next;
      return next;
    });
  }, []);

  const endDrag = useCallback((): { wasDrag: boolean } => {
    const start = dragStartRef.current;
    const history = pointerHistoryRef.current;
    const pigId = dragIdRef.current;

    dragIdRef.current = null;
    dragStartRef.current = null;
    pointerHistoryRef.current = [];

    if (!start || !pigId) return { wasDrag: false };

    const last = history[history.length - 1];
    if (!last) return { wasDrag: false };

    const dx = last.x - start.x;
    const dy = last.y - start.y;
    const moved = Math.sqrt(dx * dx + dy * dy);

    if (moved < DRAG_THRESHOLD) return { wasDrag: false };

    const { vx, vy } = computeTossVelocity(history, TOSS_VELOCITY_WINDOW_MS, last.t);
    setPigs((prev) => {
      const next = prev.map((p) => (p.id === pigId ? { ...p, vx, vy } : p));
      pigsRef.current = next;
      // Restore narrow rects immediately so hit-test is precise again.
      syncRects(next, false, displaySpaceRef.current, animalScalesRef.current);
      return next;
    });

    return { wasDrag: true };
  }, []);

  // Sync pig list to Animals: add spawns for new, remove for deleted.
  useEffect(() => {
    const now = performance.now();

    setPigs((prev) => {
      const prevMap = new Map(prev.map((p) => [p.id, p]));
      const fallbackIds = spawnedFromFallbackRef.current;
      const nextFallbackIds = new Set<string>();
      const next = roster.map((animal) => {
        const existing = prevMap.get(animal.id);
        const usingFallback = displaySpace === fallbackDisplaySpaceRef.current;
        if (existing && (!fallbackIds.has(animal.id) || usingFallback)) {
          if (fallbackIds.has(animal.id)) nextFallbackIds.add(animal.id);
          const named =
            existing.name === animal.name ? existing : { ...existing, name: animal.name };
          return animal.resting
            ? restRanchAnimal(named, regionsFor(animal.pen?.id, penRects, displaySpace))
            : named;
        }

        const pig = initPig(
          animal,
          spawnRegionFor(animal.pen?.id ?? null, penRects, displaySpace),
          now,
        );
        if (usingFallback) {
          nextFallbackIds.add(animal.id);
        }
        if (animal.resting) {
          return restRanchAnimal(pig, regionsFor(animal.pen?.id, penRects, displaySpace));
        }
        return pig;
      });
      spawnedFromFallbackRef.current = nextFallbackIds;
      pigsRef.current = next;
      return next;
    });
  }, [roster, displaySpace, penRects]);

  // Subscribe to gather-pigs / display-space events from Rust.
  // Fall back to a no-op unsubscribe if subscribe rejects so cleanup never throws.
  useEffect(() => {
    const unsubPromise = subscribeGatherPigs(gather).catch(() => () => {});
    return () => {
      unsubPromise.then((unsub) => unsub());
    };
  }, [gather]);

  useEffect(() => {
    const unsubPromise = subscribeDisplaySpace(setDisplaySpace).catch(() => () => {});
    return () => {
      unsubPromise.then((unsub) => unsub());
    };
  }, [setDisplaySpace]);

  // rAF movement loop
  useEffect(() => {
    const restingIds = new Set(
      roster.filter((animal) => animal.resting).map((animal) => animal.id),
    );

    const loop = (now: number) => {
      const dt = Math.min(now - lastTimeRef.current, 100);
      lastTimeRef.current = now;

      const displaySpace = displaySpaceRef.current;
      const updated = pigsRef.current.map((p) => {
        // Skip tick for dragged pig — position is driven by pointer events.
        if (p.id === dragIdRef.current) return p;
        const regions = regionsFor(
          penIdByAnimalRef.current.get(p.id),
          penRectsRef.current,
          displaySpace,
        );
        if (restingIds.has(p.id)) return restRanchAnimal(p, regions);
        return advanceRanchAnimal({
          animal: p,
          regions: regionsFor(
            penIdByAnimalRef.current.get(p.id),
            penRectsRef.current,
            displaySpace,
          ),
          dtMs: dt,
          nowMs: now,
          frozen: p.id === selectedIdRef.current,
          random: Math.random,
        });
      });
      pigsRef.current = updated;
      setPigs(updated);

      frameCountRef.current += 1;
      if (frameCountRef.current % RECT_UPDATE_EVERY === 0) {
        const wide = selectedIdRef.current !== null || dragIdRef.current !== null;
        syncRects(updated, wide, displaySpace, animalScalesRef.current);
      }

      rafRef.current = requestAnimationFrame(loop);
    };

    rafRef.current = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafRef.current);
  }, [roster]);

  return { pigs, pens, startDrag, moveDrag, endDrag, setDragActive };
}
