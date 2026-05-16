import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTauriFocusReader } from "./tauriFocusReader";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("tauriFocusReader", () => {
  it("preserves timer data from Rust focuses", async () => {
    mockInvoke.mockResolvedValueOnce([
      {
        id: "focus-1",
        title: "Timed focus",
        description: "",
        created_at: "",
        tasks: [],
        timer: { duration_secs: 480, started_at: 1_700_000_000, status: "Running" },
      },
    ]);

    const focuses = await createTauriFocusReader().read();

    expect(mockInvoke).toHaveBeenCalledWith("list_focuses");
    expect(focuses[0]?.timer).toEqual({
      duration_secs: 480,
      started_at: 1_700_000_000,
      status: "Running",
    });
  });
});
