export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
}

export interface RectSize {
  readonly w: number;
  readonly h: number;
}

export interface DisplaySpace {
  readonly span: RectSize;
  readonly spawnRegion: Rect;
  readonly movementRegions: readonly Rect[];
  readonly hitTestScale: number;
}
