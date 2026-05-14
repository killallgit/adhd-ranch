import type { DisplaySpace, Rect } from "../types/display";

export const RANCH_ANIMAL_SIZE = 48;
export const RANCH_EDGE_MARGIN = 60;
export const RANCH_ANIMAL_SPEED = 60;
export const RANCH_FRICTION = 0.97;
const MIN_SPEED_FRAC = 0.35;
const FRAME_INTERVAL = 150;
const MIN_TURN_MS = 3000;
const MAX_TURN_MS = 8000;

export type RanchAnimalDirection = "front" | "right" | "back" | "left";

export interface RanchAnimalState {
  readonly id: string;
  readonly name: string;
  readonly x: number;
  readonly y: number;
  readonly vx: number;
  readonly vy: number;
  readonly frameIndex: number;
  readonly direction: RanchAnimalDirection;
  readonly lastFrameAt: number;
  readonly nextTurnAt: number;
}

export interface AdvanceRanchAnimalInput {
  readonly animal: RanchAnimalState;
  readonly displaySpace: DisplaySpace;
  readonly dtMs: number;
  readonly nowMs: number;
  readonly frozen: boolean;
  readonly random: () => number;
}

export function advanceRanchAnimal({
  animal,
  displaySpace,
  dtMs,
  nowMs,
  random,
  frozen,
}: AdvanceRanchAnimalInput): RanchAnimalState {
  if (frozen) {
    return {
      ...animal,
      lastFrameAt: animal.lastFrameAt + dtMs,
      nextTurnAt: animal.nextTurnAt + dtMs,
    };
  }

  const steeringRegion = nearestRegion(animal.x, animal.y, displaySpace.movementRegions);
  let { nextTurnAt } = animal;
  let { frameIndex, lastFrameAt } = animal;
  let vx = animal.vx * RANCH_FRICTION;
  let vy = animal.vy * RANCH_FRICTION;

  const speed = Math.hypot(vx, vy);
  const minSpeed = RANCH_ANIMAL_SPEED * MIN_SPEED_FRAC;
  if (speed < minSpeed && speed > 0) {
    const scale = minSpeed / speed;
    vx *= scale;
    vy *= scale;
  } else if (speed === 0) {
    const angle = random() * 2 * Math.PI;
    vx = Math.cos(angle) * minSpeed;
    vy = Math.sin(angle) * minSpeed;
  }

  if (nowMs >= nextTurnAt) {
    const angle = random() * 2 * Math.PI;
    vx = Math.cos(angle) * RANCH_ANIMAL_SPEED;
    vy = Math.sin(angle) * RANCH_ANIMAL_SPEED;
    nextTurnAt = nowMs + MIN_TURN_MS + random() * (MAX_TURN_MS - MIN_TURN_MS);
  }

  let x = animal.x + vx * (dtMs / 1000);
  let y = animal.y + vy * (dtMs / 1000);

  if (steeringRegion) {
    const minX = steeringRegion.x;
    const maxX = steeringRegion.x + steeringRegion.w - RANCH_ANIMAL_SIZE;
    const minY = steeringRegion.y;
    const maxY = steeringRegion.y + steeringRegion.h - RANCH_ANIMAL_SIZE;
    if (animal.x < minX + RANCH_EDGE_MARGIN) vx = Math.abs(vx);
    if (animal.x > maxX - RANCH_EDGE_MARGIN) vx = -Math.abs(vx);
    if (animal.y < minY + RANCH_EDGE_MARGIN) vy = Math.abs(vy);
    if (animal.y > maxY - RANCH_EDGE_MARGIN) vy = -Math.abs(vy);
    x = animal.x + vx * (dtMs / 1000);
    y = animal.y + vy * (dtMs / 1000);
  }

  const point = nearestValidPoint(x, y, displaySpace.movementRegions);
  if (point.x !== x) {
    vx = x > point.x ? -Math.abs(vx) : Math.abs(vx);
  }
  if (point.y !== y) {
    vy = y > point.y ? -Math.abs(vy) : Math.abs(vy);
  }
  x = point.x;
  y = point.y;

  if (nowMs - lastFrameAt >= FRAME_INTERVAL) {
    frameIndex = (frameIndex + 1) % 4;
    lastFrameAt = nowMs;
  }

  return {
    ...animal,
    x,
    y,
    vx,
    vy,
    frameIndex,
    direction: direction4(vx, vy),
    lastFrameAt,
    nextTurnAt,
  };
}

function nearestRegion(x: number, y: number, regions: readonly Rect[]): Rect | null {
  if (regions.length === 0) return null;

  let best = regions[0];
  let bestPoint = clampPointToRegion(x, y, best);
  let bestDistance = squaredDistance(x, y, bestPoint.x, bestPoint.y);

  for (const region of regions.slice(1)) {
    const point = clampPointToRegion(x, y, region);
    const distance = squaredDistance(x, y, point.x, point.y);
    if (distance < bestDistance) {
      best = region;
      bestPoint = point;
      bestDistance = distance;
    }
  }

  return best;
}

function direction4(vx: number, vy: number): RanchAnimalDirection {
  if (Math.abs(vx) >= Math.abs(vy)) return vx >= 0 ? "right" : "left";
  return vy >= 0 ? "front" : "back";
}

function nearestValidPoint(
  x: number,
  y: number,
  regions: readonly Rect[],
): { x: number; y: number } {
  if (regions.length === 0) return { x, y };

  let best = clampPointToRegion(x, y, regions[0]);
  let bestDistance = squaredDistance(x, y, best.x, best.y);

  for (const region of regions.slice(1)) {
    const candidate = clampPointToRegion(x, y, region);
    const distance = squaredDistance(x, y, candidate.x, candidate.y);
    if (distance < bestDistance) {
      best = candidate;
      bestDistance = distance;
    }
  }

  return best;
}

function clampPointToRegion(x: number, y: number, region: Rect): { x: number; y: number } {
  return {
    x: clamp(x, region.x, region.x + region.w - RANCH_ANIMAL_SIZE),
    y: clamp(y, region.y, region.y + region.h - RANCH_ANIMAL_SIZE),
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function squaredDistance(ax: number, ay: number, bx: number, by: number): number {
  const dx = ax - bx;
  const dy = ay - by;
  return dx * dx + dy * dy;
}
