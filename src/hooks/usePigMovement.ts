import { useCallback, useEffect, useRef, useState } from "react";
import {
  setPigDragActive,
  subscribeDisplaySpace,
  subscribeGatherPigs,
  updatePigRects,
} from "../api/pig";
import {
  RANCH_ANIMAL_SIZE,
  RANCH_ANIMAL_SPEED,
  type RanchAnimalDirection,
  type RanchAnimalState,
  advanceRanchAnimal,
} from "../lib/ranchAnimalMovement";
import type { DisplaySpace } from "../types/display";
import type { Focus } from "../types/focus";
import type { PigHitRect } from "../types/pig";

export type { DisplaySpace, PigHitRect };

export const PIG_SIZE = RANCH_ANIMAL_SIZE;
export const HITBOX_PADDING = 16;
export const DRAG_THRESHOLD = 4;
export const TOSS_VELOCITY_WINDOW_MS = 80;

export const PIG_SPEED = RANCH_ANIMAL_SPEED; // px/s — fast enough to look alive across large spans
const EDGE_MARGIN = 60; // px from screen edge

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
  startDrag: (pigId: string, x: number, y: number) => void;
  moveDrag: (x: number, y: number) => void;
  endDrag: () => { wasDrag: boolean };
  setDragActive: (active: boolean) => void;
}

function direction4(vx: number, vy: number): PigDirection {
  if (Math.abs(vx) >= Math.abs(vy)) return vx >= 0 ? "right" : "left";
  return vy >= 0 ? "front" : "back";
}

function initPig(focus: Focus, displaySpace: DisplaySpace, now: number): PigState {
  const region = displaySpace.spawnRegion;
  const angle = Math.random() * 2 * Math.PI;
  const vx = Math.cos(angle) * PIG_SPEED;
  const vy = Math.sin(angle) * PIG_SPEED;
  return {
    id: focus.id,
    name: focus.title,
    x: region.x + EDGE_MARGIN + Math.random() * Math.max(0, region.w - 2 * EDGE_MARGIN - PIG_SIZE),
    y: region.y + EDGE_MARGIN + Math.random() * Math.max(0, region.h - 2 * EDGE_MARGIN - PIG_SIZE),
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
  focuses: readonly Focus[],
  selectedId: string | null,
  animalScales: ReadonlyMap<string, number> = new Map(),
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
  const animalScalesRef = useRef<ReadonlyMap<string, number>>(animalScales);
  const spawnedFromFallbackRef = useRef<Set<string>>(new Set());

  // Drag state — refs to avoid stale closures in the rAF loop.
  const dragIdRef = useRef<string | null>(null);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const pointerHistoryRef = useRef<PointerSample[]>([]);

  // Keep selectedId ref in sync so the rAF loop sees the latest value without restarting.
  selectedIdRef.current = selectedId;
  animalScalesRef.current = animalScales;

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

  // Sync pig list to focuses: add spawns for new, remove for deleted.
  useEffect(() => {
    const now = performance.now();

    setPigs((prev) => {
      const prevMap = new Map(prev.map((p) => [p.id, p]));
      const fallbackIds = spawnedFromFallbackRef.current;
      const nextFallbackIds = new Set<string>();
      const next = focuses.map((f) => {
        const existing = prevMap.get(f.id);
        const usingFallback = displaySpace === fallbackDisplaySpaceRef.current;
        if (existing && (!fallbackIds.has(f.id) || usingFallback)) {
          if (fallbackIds.has(f.id)) nextFallbackIds.add(f.id);
          return existing;
        }

        const pig = initPig(f, displaySpace, now);
        if (usingFallback) {
          nextFallbackIds.add(f.id);
        }
        return pig;
      });
      spawnedFromFallbackRef.current = nextFallbackIds;
      pigsRef.current = next;
      return next;
    });
  }, [focuses, displaySpace]);

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
    const loop = (now: number) => {
      const dt = Math.min(now - lastTimeRef.current, 100);
      lastTimeRef.current = now;

      const displaySpace = displaySpaceRef.current;
      const updated = pigsRef.current.map((p) => {
        // Skip tick for dragged pig — position is driven by pointer events.
        if (p.id === dragIdRef.current) return p;
        return advanceRanchAnimal({
          animal: p,
          displaySpace,
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
  }, []);

  return { pigs, startDrag, moveDrag, endDrag, setDragActive };
}
