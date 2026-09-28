import type { CSSProperties } from "react";
import type { AnimalRegionLayout } from "../types/generated/AnimalRegionLayout";

export interface RegionBoxProps {
  readonly layout: AnimalRegionLayout;
}

export function RegionBox({ layout }: RegionBoxProps) {
  const { rect, hue, label } = layout;
  const style = {
    left: rect.x,
    top: rect.y,
    width: rect.w,
    height: rect.h,
    "--region-hue": String(hue),
  } as CSSProperties;

  return (
    <div className="region-box" style={style}>
      <span className="region-box-label">{label}</span>
    </div>
  );
}
