import { useEffect, useRef, useState } from "react";
import type { RenderSceneReader } from "../api/tauriRenderSceneReader";
import type { DisplaySpace } from "../types/display";
import type { RenderScene } from "../types/generated/RenderScene";

const EMPTY_SCENE: RenderScene = { animals: [], regions: [], speciesProfiles: [] };

export interface RenderSceneState {
  readonly scene: RenderScene;
  readonly displaySpace: DisplaySpace;
}

export function useRenderScene(reader: RenderSceneReader): RenderSceneState {
  const [displaySpace, setDisplaySpace] = useState(defaultDisplaySpace);
  const displaySpaceRef = useRef(displaySpace);
  const [scene, setScene] = useState<RenderScene>(EMPTY_SCENE);

  useEffect(() => {
    let cancelled = false;
    let latestRequest = 0;

    const refresh = (space: DisplaySpace, retry = true) => {
      const request = ++latestRequest;
      reader
        .read(space.spawnRegion)
        .then((next) => {
          if (!cancelled && request === latestRequest) setScene(next);
        })
        .catch(() => {
          if (!cancelled && request === latestRequest && retry) refresh(space, false);
        });
    };

    refresh(displaySpaceRef.current);
    const unsubscribe = reader
      .subscribe((invalidation) => {
        if (invalidation.event === "display-space") {
          displaySpaceRef.current = invalidation.displaySpace;
          setDisplaySpace(invalidation.displaySpace);
          refresh(invalidation.displaySpace);
          return;
        }
        refresh(displaySpaceRef.current);
      })
      .catch(() => () => {});

    return () => {
      cancelled = true;
      unsubscribe.then((unlisten) => unlisten());
    };
  }, [reader]);

  return { scene, displaySpace };
}

function defaultDisplaySpace(): DisplaySpace {
  const w = document.documentElement.clientWidth || window.screen.width;
  const h = document.documentElement.clientHeight || window.screen.height;
  const area = { x: 0, y: 0, w, h };
  return {
    span: { w, h },
    spawnRegion: area,
    movementRegions: [area],
    hitTestScale: window.devicePixelRatio || 1,
  };
}
