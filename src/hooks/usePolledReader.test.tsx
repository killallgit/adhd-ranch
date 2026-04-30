import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { type PolledReader, usePolledReader } from "./usePolledReader";

describe("usePolledReader", () => {
  it("transitions loading -> ready with the resolved value", async () => {
    const reader: PolledReader<number> = {
      read: () => Promise.resolve(42),
    };
    const { result } = renderHook(() => usePolledReader(reader));
    expect(result.current.status).toBe("loading");
    await waitFor(() => {
      expect(result.current.status).toBe("ready");
    });
    if (result.current.status === "ready") {
      expect(result.current.value).toBe(42);
    }
  });

  it("transitions to error when reader rejects", async () => {
    const reader: PolledReader<number> = {
      read: () => Promise.reject(new Error("boom")),
    };
    const { result } = renderHook(() => usePolledReader(reader));
    await waitFor(() => {
      expect(result.current.status).toBe("error");
    });
    if (result.current.status === "error") {
      expect(result.current.error.message).toBe("boom");
    }
  });

  it("re-reads when subscribe callback fires", async () => {
    let callCount = 0;
    let trigger: (() => void) | null = null;
    const reader: PolledReader<number> = {
      read: () => {
        callCount += 1;
        return Promise.resolve(callCount);
      },
      subscribe: (onChange) => {
        trigger = onChange;
        return () => {
          trigger = null;
        };
      },
    };

    const { result } = renderHook(() => usePolledReader(reader));
    await waitFor(() => {
      expect(result.current.status).toBe("ready");
    });
    if (result.current.status === "ready") {
      expect(result.current.value).toBe(1);
    }

    await act(async () => {
      trigger?.();
    });

    await waitFor(() => {
      if (result.current.status !== "ready") throw new Error("not ready");
      expect(result.current.value).toBe(2);
    });
  });
});
