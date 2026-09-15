import type { Animal } from "../types/animal";
import type { PolledReader } from "./polledReader";

export function createFixtureAnimalReader(
  animals: readonly Animal[],
): PolledReader<readonly Animal[]> {
  return {
    read: () => Promise.resolve(animals),
  };
}
