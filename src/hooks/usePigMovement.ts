import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import type { Focus } from "../types/focus";

export const PIG_SIZE = 48;

const PIG_SPEED = 35; // px/s
const EDGE_MARGIN = 60; // px from screen edge
const FRAME_INTERVAL = 150; // ms per animation frame
const MIN_TURN_MS = 3000;
const MAX_TURN_MS = 8000;
const RECT_UPDATE_EVERY = 4; // rAF frames between pig-rect syncs to Rust

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

function direction4(vx: number, vy: number): PigDirection {
  if (Math.abs(vx) >= Math.abs(vy)) return vx >= 0 ? "right" : "left";
  return vy >= 0 ? "front" : "back";
}

function randomTurnDelay(): number {
  return MIN_TURN_MS + Math.random() * (MAX_TURN_MS - MIN_TURN_MS);
}

function initPig(focus: Focus, screenW: number, screenH: number, now: number): PigState {
  const angle = Math.random() * 2 * Math.PI;
  const vx = Math.cos(angle) * PIG_SPEED;
  const vy = Math.sin(angle) * PIG_SPEED;
  return {
    id: focus.id,
    name: focus.title,
    x: EDGE_MARGIN + Math.random() * (screenW - 2 * EDGE_MARGIN - PIG_SIZE),
    y: EDGE_MARGIN + Math.random() * (screenH - 2 * EDGE_MARGIN - PIG_SIZE),
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
): PigState {
  if (frozen) return pig;
  let { x, y, vx, vy, frameIndex, lastFrameAt, nextTurnAt, direction } = pig;

  // Random direction change
  if (now >= nextTurnAt) {
    const angle = Math.random() * 2 * Math.PI;
    vx = Math.cos(angle) * PIG_SPEED;
    vy = Math.sin(angle) * PIG_SPEED;
    nextTurnAt = now + randomTurnDelay();
  }

  // Soft boundary steering: blend away-from-edge velocity when within margin
  if (x < EDGE_MARGIN) vx = Math.abs(vx) || PIG_SPEED * 0.5;
  if (x > screenW - EDGE_MARGIN - PIG_SIZE) vx = -(Math.abs(vx) || PIG_SPEED * 0.5);
  if (y < EDGE_MARGIN) vy = Math.abs(vy) || PIG_SPEED * 0.5;
  if (y > screenH - EDGE_MARGIN - PIG_SIZE) vy = -(Math.abs(vy) || PIG_SPEED * 0.5);

  // Clamp speed
  const speed = Math.sqrt(vx * vx + vy * vy);
  if (speed > PIG_SPEED * 1.2) {
    vx = (vx / speed) * PIG_SPEED;
    vy = (vy / speed) * PIG_SPEED;
  }

  // Update position and reflect velocity at hard boundaries so pigs never escape.
  x += vx * (dt / 1000);
  y += vy * (dt / 1000);
  if (x < 0) { x = 0; vx = Math.abs(vx); }
  if (x > screenW - PIG_SIZE) { x = screenW - PIG_SIZE; vx = -Math.abs(vx); }
  if (y < 0) { y = 0; vy = Math.abs(vy); }
  if (y > screenH - PIG_SIZE) { y = screenH - PIG_SIZE; vy = -Math.abs(vy); }

  direction = direction4(vx, vy);

  if (now - lastFrameAt >= FRAME_INTERVAL) {
    frameIndex = (frameIndex + 1) % 4;
    lastFrameAt = now;
  }

  return { ...pig, x, y, vx, vy, frameIndex, direction, lastFrameAt, nextTurnAt };
}

function syncRects(pigs: PigState[]): void {
  const dpr = window.devicePixelRatio || 1;
  const rects = pigs.map((p) => ({
    x: p.x * dpr,
    y: p.y * dpr,
    size: PIG_SIZE * dpr,
  }));
  invoke("update_pig_rects", { rects }).catch(() => {
    // Silently ignore when running outside Tauri (e.g., browser dev)
  });
}

export function usePigMovement(focuses: readonly Focus[], selectedId: string | null): PigState[] {
  const [pigs, setPigs] = useState<PigState[]>([]);
  const pigsRef = useRef<PigState[]>([]);
  const selectedIdRef = useRef<string | null>(selectedId);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(performance.now());
  const frameCountRef = useRef<number>(0);

  // Keep ref in sync so the rAF loop sees the latest value without restarting.
  selectedIdRef.current = selectedId;

  // Sync pig list to focuses: add spawns for new, remove for deleted.
  useEffect(() => {
    const screenW = window.innerWidth || window.screen.width;
    const screenH = window.innerHeight || window.screen.height;
    const now = performance.now();

    setPigs((prev) => {
      const prevMap = new Map(prev.map((p) => [p.id, p]));
      const next = focuses.map((f) => prevMap.get(f.id) ?? initPig(f, screenW, screenH, now));
      pigsRef.current = next;
      return next;
    });
  }, [focuses]);

  // rAF movement loop
  useEffect(() => {
    const loop = (now: number) => {
      const dt = Math.min(now - lastTimeRef.current, 100);
      lastTimeRef.current = now;

      const screenW = window.innerWidth || window.screen.width;
      const screenH = window.innerHeight || window.screen.height;

      const updated = pigsRef.current.map((p) =>
        tickPig(p, dt, now, screenW, screenH, p.id === selectedIdRef.current),
      );
      pigsRef.current = updated;
      setPigs(updated);

      frameCountRef.current += 1;
      if (frameCountRef.current % RECT_UPDATE_EVERY === 0) {
        syncRects(updated);
      }

      rafRef.current = requestAnimationFrame(loop);
    };

    rafRef.current = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafRef.current);
  }, []);

  return pigs;
}
