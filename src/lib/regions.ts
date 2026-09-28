import type { DisplaySpace } from "../types/display";
import type { Animal } from "../types/generated/Animal";
import type { AnimalRegionLayout } from "../types/generated/AnimalRegionLayout";
import type { RenderRect } from "../types/generated/RenderRect";

export function movementRegionsFor(
  animal: Animal,
  layouts: readonly AnimalRegionLayout[],
  displaySpace: DisplaySpace,
): readonly RenderRect[] {
  const rect = assignedRegion(animal, layouts);
  return rect ? [rect] : displaySpace.movementRegions;
}

export function spawnRegionFor(
  animal: Animal,
  layouts: readonly AnimalRegionLayout[],
  displaySpace: DisplaySpace,
): RenderRect {
  return assignedRegion(animal, layouts) ?? displaySpace.spawnRegion;
}

function assignedRegion(animal: Animal, layouts: readonly AnimalRegionLayout[]): RenderRect | null {
  if (animal.regionId === null) return null;
  return layouts.find((layout) => layout.id === animal.regionId)?.rect ?? null;
}
