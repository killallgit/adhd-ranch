import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import { useFocuses } from "./useFocuses";

describe("useFocuses", () => {
  it("transitions loading → ready with focuses", async () => {
    const reader = createFixtureFocusReader([{ id: "a", title: "A", description: "", tasks: [] }]);
    const { result } = renderHook(() => useFocuses(reader));
    expect(result.current.status).toBe("loading");
    await waitFor(() => {
      expect(result.current.status).toBe("ready");
    });
    if (result.current.status === "ready") {
      expect(result.current.focuses).toHaveLength(1);
      expect(result.current.focuses[0]?.id).toBe("a");
    }
  });
});
