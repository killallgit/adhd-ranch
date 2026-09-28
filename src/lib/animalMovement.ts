import type { Animal } from "../types/generated/Animal";
import type { Motion } from "../types/generated/Motion";
import type { RenderRect } from "../types/generated/RenderRect";
import type { SpeciesProfile } from "../types/generated/SpeciesProfile";

const EDGE_MARGIN_PX = 60;

export type AnimalDirection = "front" | "right" | "back" | "left";

export interface AnimalMovementState {
  readonly id: string;
  readonly label: string;
  readonly x: number;
  readonly y: number;
  readonly vx: number;
  readonly vy: number;
  readonly frameIndex: number;
  readonly direction: AnimalDirection;
  readonly lastFrameAt: number;
  readonly nextTurnAt: number;
}

export interface PointerSample {
  readonly x: number;
  readonly y: number;
  readonly t: number;
}

export interface AdvanceAnimalInput {
  readonly animal: AnimalMovementState;
  readonly regions: readonly RenderRect[];
  readonly dtMs: number;
  readonly nowMs: number;
  readonly motion: Motion;
  readonly frozen: boolean;
  readonly random: () => number;
  readonly profile: SpeciesProfile;
}

export function spawnAnimal(
  animal: Animal,
  region: RenderRect,
  profile: SpeciesProfile,
  nowMs: number,
  random: () => number,
): AnimalMovementState {
  const margin = edgeMargin(region);
  const angle = random() * 2 * Math.PI;
  const vx = Math.cos(angle) * profile.walkSpeedPxPerSecond;
  const vy = Math.sin(angle) * profile.walkSpeedPxPerSecond;
  return {
    id: animal.id,
    label: animal.label,
    x:
      region.x +
      margin +
      random() * Math.max(0, region.w - 2 * margin - profile.movementFootprintPx),
    y:
      region.y +
      margin +
      random() * Math.max(0, region.h - 2 * margin - profile.movementFootprintPx),
    vx,
    vy,
    frameIndex: 0,
    direction: direction4(vx, vy),
    lastFrameAt: nowMs,
    nextTurnAt: nowMs + profile.turnMinMs + random() * (profile.turnMaxMs - profile.turnMinMs),
  };
}

export function reconcileAnimalMovement(
  existing: AnimalMovementState | undefined,
  animal: Animal,
  spawnRegion: RenderRect,
  profile: SpeciesProfile,
  nowMs: number,
  random: () => number,
): AnimalMovementState {
  if (!existing) return spawnAnimal(animal, spawnRegion, profile, nowMs, random);
  return existing.label === animal.label ? existing : { ...existing, label: animal.label };
}

export function advanceAnimal({
  animal,
  regions,
  dtMs,
  nowMs,
  motion,
  frozen,
  random,
  profile,
}: AdvanceAnimalInput): AnimalMovementState {
  if (frozen) {
    return {
      ...animal,
      lastFrameAt: animal.lastFrameAt + dtMs,
      nextTurnAt: animal.nextTurnAt + dtMs,
    };
  }
  if (motion === "resting") return restAnimal(animal, regions, profile);

  const steeringRegion = nearestRegion(animal.x, animal.y, regions, profile);
  let { nextTurnAt } = animal;
  let { frameIndex, lastFrameAt } = animal;
  let vx = animal.vx * profile.friction;
  let vy = animal.vy * profile.friction;

  const speed = Math.hypot(vx, vy);
  const minimumSpeed = profile.walkSpeedPxPerSecond * profile.minimumSpeedFraction;
  if (speed < minimumSpeed && speed > 0) {
    const scale = minimumSpeed / speed;
    vx *= scale;
    vy *= scale;
  } else if (speed === 0) {
    const angle = random() * 2 * Math.PI;
    vx = Math.cos(angle) * minimumSpeed;
    vy = Math.sin(angle) * minimumSpeed;
  }

  if (nowMs >= nextTurnAt) {
    const angle = random() * 2 * Math.PI;
    vx = Math.cos(angle) * profile.walkSpeedPxPerSecond;
    vy = Math.sin(angle) * profile.walkSpeedPxPerSecond;
    nextTurnAt = nowMs + profile.turnMinMs + random() * (profile.turnMaxMs - profile.turnMinMs);
  }

  let x = animal.x + vx * (dtMs / 1_000);
  let y = animal.y + vy * (dtMs / 1_000);

  if (steeringRegion) {
    const margin = edgeMargin(steeringRegion);
    const minX = steeringRegion.x;
    const maxX = steeringRegion.x + steeringRegion.w - profile.movementFootprintPx;
    const minY = steeringRegion.y;
    const maxY = steeringRegion.y + steeringRegion.h - profile.movementFootprintPx;
    if (animal.x < minX + margin) vx = Math.abs(vx);
    if (animal.x > maxX - margin) vx = -Math.abs(vx);
    if (animal.y < minY + margin) vy = Math.abs(vy);
    if (animal.y > maxY - margin) vy = -Math.abs(vy);
    x = animal.x + vx * (dtMs / 1_000);
    y = animal.y + vy * (dtMs / 1_000);
  }

  const point = nearestValidPoint(x, y, regions, profile);
  if (point.x !== x) vx = x > point.x ? -Math.abs(vx) : Math.abs(vx);
  if (point.y !== y) vy = y > point.y ? -Math.abs(vy) : Math.abs(vy);

  if (nowMs - lastFrameAt >= profile.frameIntervalMs) {
    frameIndex = (frameIndex + 1) % 4;
    lastFrameAt = nowMs;
  }

  return {
    ...animal,
    x: point.x,
    y: point.y,
    vx,
    vy,
    frameIndex,
    direction: direction4(vx, vy),
    lastFrameAt,
    nextTurnAt,
  };
}

function restAnimal(
  animal: AnimalMovementState,
  regions: readonly RenderRect[],
  profile: SpeciesProfile,
): AnimalMovementState {
  const point = nearestValidPoint(animal.x, animal.y, regions, profile);
  if (
    animal.vx === 0 &&
    animal.vy === 0 &&
    animal.direction === "back" &&
    point.x === animal.x &&
    point.y === animal.y
  ) {
    return animal;
  }
  return { ...animal, x: point.x, y: point.y, vx: 0, vy: 0, direction: "back" };
}

export function computeTossVelocity(
  samples: readonly PointerSample[],
  windowMs: number,
  nowMs: number,
  profile: SpeciesProfile,
): { vx: number; vy: number } {
  const recent = samples.filter((sample) => sample.t >= nowMs - windowMs);
  if (recent.length < 2) return { vx: 0, vy: 0 };
  const first = recent[0];
  const last = recent[recent.length - 1];
  if (!first || !last || last.t === first.t) return { vx: 0, vy: 0 };

  const elapsedMs = last.t - first.t;
  const vx = ((last.x - first.x) / elapsedMs) * 1_000;
  const vy = ((last.y - first.y) / elapsedMs) * 1_000;
  const maximumSpeed = profile.walkSpeedPxPerSecond * profile.maximumTossSpeedMultiplier;
  const speed = Math.hypot(vx, vy);
  if (speed <= maximumSpeed) return { vx, vy };
  return { vx: (vx / speed) * maximumSpeed, vy: (vy / speed) * maximumSpeed };
}

function edgeMargin(region: RenderRect): number {
  return Math.min(EDGE_MARGIN_PX, region.w / 4, region.h / 4);
}

function nearestRegion(
  x: number,
  y: number,
  regions: readonly RenderRect[],
  profile: SpeciesProfile,
): RenderRect | null {
  if (regions.length === 0) return null;
  let best = regions[0];
  let bestPoint = clampPointToRegion(x, y, best, profile);
  let bestDistance = squaredDistance(x, y, bestPoint.x, bestPoint.y);
  for (const region of regions.slice(1)) {
    const point = clampPointToRegion(x, y, region, profile);
    const distance = squaredDistance(x, y, point.x, point.y);
    if (distance < bestDistance) {
      best = region;
      bestPoint = point;
      bestDistance = distance;
    }
  }
  return best;
}

function nearestValidPoint(
  x: number,
  y: number,
  regions: readonly RenderRect[],
  profile: SpeciesProfile,
): { x: number; y: number } {
  if (regions.length === 0) return { x, y };
  let best = clampPointToRegion(x, y, regions[0], profile);
  let bestDistance = squaredDistance(x, y, best.x, best.y);
  for (const region of regions.slice(1)) {
    const candidate = clampPointToRegion(x, y, region, profile);
    const distance = squaredDistance(x, y, candidate.x, candidate.y);
    if (distance < bestDistance) {
      best = candidate;
      bestDistance = distance;
    }
  }
  return best;
}

function clampPointToRegion(
  x: number,
  y: number,
  region: RenderRect,
  profile: SpeciesProfile,
): { x: number; y: number } {
  return {
    x: clamp(x, region.x, Math.max(region.x, region.x + region.w - profile.movementFootprintPx)),
    y: clamp(y, region.y, Math.max(region.y, region.y + region.h - profile.movementFootprintPx)),
  };
}

function direction4(vx: number, vy: number): AnimalDirection {
  if (Math.abs(vx) >= Math.abs(vy)) return vx >= 0 ? "right" : "left";
  return vy >= 0 ? "front" : "back";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function squaredDistance(ax: number, ay: number, bx: number, by: number): number {
  const dx = ax - bx;
  const dy = ay - by;
  return dx * dx + dy * dy;
}
