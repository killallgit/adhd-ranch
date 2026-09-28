import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { RenderSceneInvalidation, RenderSceneReader } from "../api/tauriRenderSceneReader";
import type { RenderScene } from "../types/generated/RenderScene";
import { useRenderScene } from "./useRenderScene";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function scene(label: string): RenderScene {
  return {
    animals: [
      {
        id: label,
        label,
        species: "pig",
        size: { kind: "fixed", px: 48 },
        regionId: null,
        motion: "walking",
      },
    ],
    regions: [],
    speciesProfiles: [],
  };
}

describe("useRenderScene", () => {
  it("discards an older read that resolves after a newer invalidation", async () => {
    const first = deferred<RenderScene>();
    const second = deferred<RenderScene>();
    let invalidate: ((event: RenderSceneInvalidation) => void) | undefined;
    const reader: RenderSceneReader = {
      read: vi.fn().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise),
      subscribe: vi.fn(async (callback) => {
        invalidate = callback;
        return () => {};
      }),
    };

    const { result } = renderHook(() => useRenderScene(reader));
    await waitFor(() => expect(invalidate).toBeDefined());
    act(() => invalidate?.({ event: "focuses-changed" }));
    await act(async () => second.resolve(scene("new")));
    expect(result.current.scene.animals[0]?.id).toBe("new");
    await act(async () => first.resolve(scene("old")));
    expect(result.current.scene.animals[0]?.id).toBe("new");
  });

  it("retries the latest scene read once after a transient failure", async () => {
    const reader: RenderSceneReader = {
      read: vi
        .fn()
        .mockRejectedValueOnce(new Error("transient read failure"))
        .mockResolvedValueOnce(scene("recovered")),
      subscribe: vi.fn(async () => () => {}),
    };

    const { result } = renderHook(() => useRenderScene(reader));

    await waitFor(() => expect(result.current.scene.animals[0]?.id).toBe("recovered"));
    expect(reader.read).toHaveBeenCalledTimes(2);
  });
});
