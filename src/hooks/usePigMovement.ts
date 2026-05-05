import { useCallback, useEffect, useRef, useState } from "react";
import { updatePigRects } from "../api/pig";
import type { Focus } from "../types/focus";
import type { PigHitRect, SpawnRegion } from "../types/pig";

export type { PigHitRect, SpawnRegion };

export const PIG_SIZE = 48;
export const HITBOX_PADDING = 16;
export const DRAG_THRESHOLD = 4;
export const TOSS_VELOCITY_WINDOW_MS = 80;
export const FRICTION = 0.97;

export const PIG_SPEED = 60; // px/s — fast enough to look alive across large spans
const MIN_SPEED_FRAC = 0.35; // friction floor: never drop below this fraction of PIG_SPEED
const EDGE_MARGIN = 60; // px from screen edge

const FRAME_INTERVAL = 150; // ms per animation frame
const MIN_TURN_MS = 3000;
const MAX_TURN_MS = 8000;
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

export type PigDirection = "front" | "right" | "back" | "left";

export interface PigState {
  id: string;
  name: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  frameIndex: number;
  direction: PigDirection;
  lastFrameAt: number;
  nextTurnAt: number;
}

export interface PigMovementResult {
  pigs: PigState[];
  startDrag: (pigId: string, x: number, y: number) => void;
  moveDrag: (x: number, y: number) => void;
  endDrag: () => { wasDrag: boolean };
  gather: () => void;
  setRegion: (r: SpawnRegion) => void;
}

function direction4(vx: number, vy: number): PigDirection {
  if (Math.abs(vx) >= Math.abs(vy)) return vx >= 0 ? "right" : "left";
  return vy >= 0 ? "front" : "back";
}

function randomTurnDelay(): number {
  return MIN_TURN_MS + Math.random() * (MAX_TURN_MS - MIN_TURN_MS);
}

function initPig(focus: Focus, region: SpawnRegion, now: number): PigState {
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
    nextTurnAt: now + randomTurnDelay(),
  };
}

function tickPig(
  pig: PigState,
  dt: number,
  now: number,
  screenW: number,
  screenH: number,
  frozen: boolean,
  primaryRegion: SpawnRegion,
): PigState {
  // Advance timers while frozen so nextTurnAt/lastFrameAt don't expire during
  // the pause, preventing an immediate turn or frame jump on unfreeze.
  if (frozen) {
    return {
      ...pig,
      nextTurnAt: pig.nextTurnAt + dt,
      lastFrameAt: pig.lastFrameAt + dt,
    };
  }
  let { x, y, vx, vy, frameIndex, lastFrameAt, nextTurnAt, direction } = pig;

  // Apply friction so toss velocity decelerates naturally.
  vx *= FRICTION;
  vy *= FRICTION;

  // Enforce minimum speed so pigs never look frozen between turns.
  const speed = Math.sqrt(vx * vx + vy * vy);
  const minSpeed = PIG_SPEED * MIN_SPEED_FRAC;
  if (speed < minSpeed && speed > 0) {
    const scale = minSpeed / speed;
    vx *= scale;
    vy *= scale;
  } else if (speed === 0) {
    const angle = Math.random() * 2 * Math.PI;
    vx = Math.cos(angle) * minSpeed;
    vy = Math.sin(angle) * minSpeed;
  }

  // Random direction change
  if (now >= nextTurnAt) {
    const angle = Math.random() * 2 * Math.PI;
    vx = Math.cos(angle) * PIG_SPEED;
    vy = Math.sin(angle) * PIG_SPEED;
    nextTurnAt = now + randomTurnDelay();
  }

  // Effective y ceiling: when pig is in the primary-display x-range it must stay
  // within the primary display height. Outside that x-range (e.g. portrait monitor
  // to the left) the full span height is available.
  const inPrimary = x >= primaryRegion.x && x <= primaryRegion.x + primaryRegion.w;
  const effectiveMaxY = inPrimary ? primaryRegion.y + primaryRegion.h : screenH;

  // Soft boundary steering: blend away-from-edge velocity when within margin
  if (x < EDGE_MARGIN) vx = Math.abs(vx) || PIG_SPEED * 0.5;
  if (x > screenW - EDGE_MARGIN - PIG_SIZE) vx = -(Math.abs(vx) || PIG_SPEED * 0.5);
  if (y < EDGE_MARGIN) vy = Math.abs(vy) || PIG_SPEED * 0.5;
  if (y > effectiveMaxY - EDGE_MARGIN - PIG_SIZE) vy = -(Math.abs(vy) || PIG_SPEED * 0.5);

  // Clamp to max toss speed.
  const speed2 = Math.sqrt(vx * vx + vy * vy);
  if (speed2 > PIG_SPEED * 6) {
    vx = (vx / speed2) * PIG_SPEED * 6;
    vy = (vy / speed2) * PIG_SPEED * 6;
  }

  // Update position and reflect velocity at hard boundaries so pigs never escape.
  x += vx * (dt / 1000);
  y += vy * (dt / 1000);
  if (x < 0) {
    x = 0;
    vx = Math.abs(vx);
  }
  if (x > screenW - PIG_SIZE) {
    x = screenW - PIG_SIZE;
    vx = -Math.abs(vx);
  }
  if (y < 0) {
    y = 0;
    vy = Math.abs(vy);
  }
  if (y > effectiveMaxY - PIG_SIZE) {
    y = effectiveMaxY - PIG_SIZE;
    vy = -Math.abs(vy);
  }

  direction = direction4(vx, vy);

  if (now - lastFrameAt >= FRAME_INTERVAL) {
    frameIndex = (frameIndex + 1) % 4;
    lastFrameAt = now;
  }

  return { ...pig, x, y, vx, vy, frameIndex, direction, lastFrameAt, nextTurnAt };
}

export function buildHitRects(pigs: PigState[], dpr: number): PigHitRect[] {
  return pigs.map((p) => ({
    x: (p.x - HITBOX_PADDING / 2) * dpr,
    y: (p.y - HITBOX_PADDING / 2) * dpr,
    size: (PIG_SIZE + HITBOX_PADDING) * dpr,
  }));
}

function syncRects(pigs: PigState[], wide: boolean): void {
  const dpr = window.devicePixelRatio || 1;
  // Wide rect (detail open or dragging) keeps overlay interactive across the full viewport.
  const rects = wide
    ? [{ x: 0, y: 0, size: Math.max(window.innerWidth, window.innerHeight) * dpr * 2 }]
    : buildHitRects(pigs, dpr);
  updatePigRects(rects).catch(() => {});
}

function defaultRegion(): SpawnRegion {
  const w = document.documentElement.clientWidth || window.screen.width;
  const h = document.documentElement.clientHeight || window.screen.height;
  return { x: 0, y: 0, w, h };
}

export function usePigMovement(
  focuses: readonly Focus[],
  selectedId: string | null,
): PigMovementResult {
  const [pigs, setPigs] = useState<PigState[]>([]);
  const pigsRef = useRef<PigState[]>([]);
  const selectedIdRef = useRef<string | null>(selectedId);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(performance.now());
  const frameCountRef = useRef<number>(0);
  // Region is updated via setRegion when Rust emits display-region.
  const regionRef = useRef<SpawnRegion>(defaultRegion());

  // Drag state — refs to avoid stale closures in the rAF loop.
  const dragIdRef = useRef<string | null>(null);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const pointerHistoryRef = useRef<PointerSample[]>([]);

  // Keep selectedId ref in sync so the rAF loop sees the latest value without restarting.
  selectedIdRef.current = selectedId;

  const setRegion = useCallback((r: SpawnRegion) => {
    regionRef.current = r;
  }, []);

  const startDrag = useCallback((pigId: string, x: number, y: number) => {
    dragIdRef.current = pigId;
    dragStartRef.current = { x, y };
    pointerHistoryRef.current = [{ x, y, t: performance.now() }];
    // Widen hit-rect immediately so the overlay stays interactive during the drag.
    // Without this there is a ~67ms window where the window is click-through,
    // which breaks pointer capture when crossing monitor boundaries.
    syncRects(pigsRef.current, true);
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
      const r = regionRef.current;
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
      syncRects(next, false);
      return next;
    });

    return { wasDrag: true };
  }, []);

  // Sync pig list to focuses: add spawns for new, remove for deleted.
  useEffect(() => {
    const region = regionRef.current;
    const now = performance.now();

    setPigs((prev) => {
      const prevMap = new Map(prev.map((p) => [p.id, p]));
      const next = focuses.map((f) => prevMap.get(f.id) ?? initPig(f, region, now));
      pigsRef.current = next;
      return next;
    });
  }, [focuses]);

  // rAF movement loop
  useEffect(() => {
    const loop = (now: number) => {
      const dt = Math.min(now - lastTimeRef.current, 100);
      lastTimeRef.current = now;

      const screenW = document.documentElement.clientWidth || window.screen.width;
      const screenH = document.documentElement.clientHeight || window.screen.height;

      const region = regionRef.current;
      const updated = pigsRef.current.map((p) => {
        // Skip tick for dragged pig — position is driven by pointer events.
        if (p.id === dragIdRef.current) return p;
        return tickPig(p, dt, now, screenW, screenH, p.id === selectedIdRef.current, region);
      });
      pigsRef.current = updated;
      setPigs(updated);

      frameCountRef.current += 1;
      if (frameCountRef.current % RECT_UPDATE_EVERY === 0) {
        const wide = selectedIdRef.current !== null || dragIdRef.current !== null;
        syncRects(updated, wide);
      }

      rafRef.current = requestAnimationFrame(loop);
    };

    rafRef.current = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafRef.current);
  }, []);

  return { pigs, startDrag, moveDrag, endDrag, gather, setRegion };
}
