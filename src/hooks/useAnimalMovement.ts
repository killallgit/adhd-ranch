import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { setPigDragActive, subscribeGatherPigs, updatePigRects } from "../api/pig";
import {
  type AnimalMovementState,
  type PointerSample,
  advanceAnimal,
  computeTossVelocity,
  reconcileAnimalMovement,
} from "../lib/animalMovement";
import { sampleAnimalSize } from "../lib/animalSize";
import { movementRegionsFor, spawnRegionFor } from "../lib/regions";
import type { DisplaySpace } from "../types/display";
import type { RenderScene } from "../types/generated/RenderScene";
import type { SpeciesProfile } from "../types/generated/SpeciesProfile";
import type { PigHitRect } from "../types/pig";

export const HITBOX_PADDING = 16;
export const DRAG_THRESHOLD = 4;
export const TOSS_VELOCITY_WINDOW_MS = 80;
const RECT_UPDATE_EVERY = 4;

export interface AnimalMovementResult {
  readonly animals: readonly AnimalMovementState[];
  readonly startDrag: (animalId: string, x: number, y: number) => void;
  readonly moveDrag: (x: number, y: number) => void;
  readonly endDrag: () => { wasDrag: boolean };
  readonly setDragActive: (active: boolean) => void;
}

export function buildHitRects(
  animals: readonly AnimalMovementState[],
  scale: number,
  sampledSizes: ReadonlyMap<string, number>,
): PigHitRect[] {
  return animals.map((animal) => ({
    x: (animal.x - HITBOX_PADDING / 2) * scale,
    y: (animal.y - HITBOX_PADDING / 2) * scale,
    size: ((sampledSizes.get(animal.id) ?? 0) + HITBOX_PADDING) * scale,
  }));
}

export function useAnimalMovement(
  scene: RenderScene,
  displaySpace: DisplaySpace,
  frozenId: string | null,
): AnimalMovementResult {
  const [movement, setMovement] = useState<AnimalMovementState[]>([]);
  const movementRef = useRef<AnimalMovementState[]>([]);
  const sceneRef = useRef(scene);
  const displaySpaceRef = useRef(displaySpace);
  const frozenIdRef = useRef(frozenId);
  const rafRef = useRef(0);
  const lastTimeRef = useRef(performance.now());
  const frameCountRef = useRef(0);
  const dragIdRef = useRef<string | null>(null);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const pointerHistoryRef = useRef<PointerSample[]>([]);

  useLayoutEffect(() => {
    sceneRef.current = scene;
    displaySpaceRef.current = displaySpace;
    frozenIdRef.current = frozenId;
  }, [scene, displaySpace, frozenId]);

  useEffect(() => {
    const nowMs = performance.now();
    const profileBySpecies = profilesBySpecies(scene);
    const previousById = new Map(movementRef.current.map((animal) => [animal.id, animal]));
    const next = scene.animals.flatMap((animal) => {
      const profile = profileBySpecies.get(animal.species);
      if (!profile) return [];
      const regions = movementRegionsFor(animal, scene.regions, displaySpace);
      const spawnRegion = spawnRegionFor(animal, scene.regions, displaySpace);
      let state = reconcileAnimalMovement(
        previousById.get(animal.id),
        animal,
        spawnRegion,
        profile,
        nowMs,
        Math.random,
      );
      if (animal.motion === "resting") {
        state = advanceAnimal({
          animal: state,
          regions,
          dtMs: 0,
          nowMs,
          motion: animal.motion,
          frozen: false,
          random: Math.random,
          profile,
        });
      }
      return [state];
    });
    movementRef.current = next;
    setMovement(next);
  }, [scene, displaySpace]);

  const setDragActive = useCallback((active: boolean) => {
    setPigDragActive(active).catch(() => {});
  }, []);

  const startDrag = useCallback((animalId: string, x: number, y: number) => {
    dragIdRef.current = animalId;
    dragStartRef.current = { x, y };
    pointerHistoryRef.current = [{ x, y, t: performance.now() }];
    syncRects(movementRef.current, true, sceneRef.current, displaySpaceRef.current);
  }, []);

  const moveDrag = useCallback((x: number, y: number) => {
    if (!dragIdRef.current) return;
    const nowMs = performance.now();
    pointerHistoryRef.current.push({ x, y, t: nowMs });
    pointerHistoryRef.current = pointerHistoryRef.current.filter(
      (sample) => sample.t >= nowMs - 200,
    );
    const next = movementRef.current.map((animal) =>
      animal.id === dragIdRef.current ? { ...animal, x, y } : animal,
    );
    movementRef.current = next;
    setMovement(next);
  }, []);

  const endDrag = useCallback((): { wasDrag: boolean } => {
    const start = dragStartRef.current;
    const samples = pointerHistoryRef.current;
    const animalId = dragIdRef.current;
    dragIdRef.current = null;
    dragStartRef.current = null;
    pointerHistoryRef.current = [];
    if (!start || !animalId) return { wasDrag: false };
    const last = samples[samples.length - 1];
    if (!last || Math.hypot(last.x - start.x, last.y - start.y) < DRAG_THRESHOLD) {
      return { wasDrag: false };
    }

    const source = sceneRef.current.animals.find((animal) => animal.id === animalId);
    const profile = source ? profilesBySpecies(sceneRef.current).get(source.species) : undefined;
    if (profile) {
      const velocity = computeTossVelocity(samples, TOSS_VELOCITY_WINDOW_MS, last.t, profile);
      const next = movementRef.current.map((animal) =>
        animal.id === animalId ? { ...animal, ...velocity } : animal,
      );
      movementRef.current = next;
      setMovement(next);
      syncRects(next, false, sceneRef.current, displaySpaceRef.current);
    }
    return { wasDrag: true };
  }, []);

  const gather = useCallback(() => {
    const area = displaySpaceRef.current.spawnRegion;
    const profiles = profilesBySpecies(sceneRef.current);
    const sourceById = new Map(sceneRef.current.animals.map((animal) => [animal.id, animal]));
    const margin = 20;
    const next = movementRef.current.map((state, index) => {
      const source = sourceById.get(state.id);
      const profile = source ? profiles.get(source.species) : undefined;
      if (!profile) return state;
      const footprint = profile.movementFootprintPx;
      const rowHeight = footprint + 24;
      const columnWidth = footprint + 24;
      const rows = Math.max(1, Math.floor((area.h - margin * 2) / rowHeight));
      return {
        ...state,
        x: area.x + area.w - margin - footprint - Math.floor(index / rows) * columnWidth,
        y: area.y + margin + (index % rows) * rowHeight,
        vx: 0,
        vy: 0,
      };
    });
    movementRef.current = next;
    setMovement(next);
  }, []);

  useEffect(() => {
    const unsubscribe = subscribeGatherPigs(gather).catch(() => () => {});
    return () => {
      unsubscribe.then((unlisten) => unlisten());
    };
  }, [gather]);

  useEffect(() => {
    const loop = (nowMs: number) => {
      const elapsedMs = Math.min(nowMs - lastTimeRef.current, 100);
      lastTimeRef.current = nowMs;
      const currentScene = sceneRef.current;
      const currentDisplaySpace = displaySpaceRef.current;
      const sourceById = new Map(currentScene.animals.map((animal) => [animal.id, animal]));
      const profiles = profilesBySpecies(currentScene);
      const next = movementRef.current.map((state) => {
        if (state.id === dragIdRef.current) return state;
        const source = sourceById.get(state.id);
        if (!source) return state;
        const profile = profiles.get(source.species);
        if (!profile) return state;
        return advanceAnimal({
          animal: state,
          regions: movementRegionsFor(source, currentScene.regions, currentDisplaySpace),
          dtMs: elapsedMs,
          nowMs,
          motion: source.motion,
          frozen: state.id === frozenIdRef.current,
          random: Math.random,
          profile,
        });
      });
      movementRef.current = next;
      setMovement(next);

      frameCountRef.current += 1;
      if (frameCountRef.current % RECT_UPDATE_EVERY === 0) {
        syncRects(
          next,
          frozenIdRef.current !== null || dragIdRef.current !== null,
          currentScene,
          currentDisplaySpace,
        );
      }
      rafRef.current = requestAnimationFrame(loop);
    };

    rafRef.current = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafRef.current);
  }, []);

  return { animals: movement, startDrag, moveDrag, endDrag, setDragActive };
}

function profilesBySpecies(scene: RenderScene): ReadonlyMap<string, SpeciesProfile> {
  return new Map(scene.speciesProfiles.map((profile) => [profile.species, profile]));
}

function syncRects(
  movement: readonly AnimalMovementState[],
  wide: boolean,
  scene: RenderScene,
  displaySpace: DisplaySpace,
): void {
  const sampledSizes = new Map(
    scene.animals.map((animal) => [animal.id, sampleAnimalSize(animal.size, Date.now())]),
  );
  const scale = displaySpace.hitTestScale;
  const rects = wide
    ? [{ x: 0, y: 0, size: Math.max(displaySpace.span.w, displaySpace.span.h) * scale * 2 }]
    : buildHitRects(movement, scale, sampledSizes);
  updatePigRects(rects).catch(() => {});
}
