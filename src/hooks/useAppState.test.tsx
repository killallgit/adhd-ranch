import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { type Caps, DEFAULT_CAPS, createFixtureCapsReader } from "../api/caps";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import { createFixtureProposalReader } from "../api/fixtureProposalReader";
import type { PolledReader } from "../hooks/usePolledReader";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";
import { useAppState } from "./useAppState";

const fixtureCaps: Caps = { max_focuses: 3, max_tasks_per_focus: 4 };

function failingFocusReader(error: Error): PolledReader<readonly Focus[]> {
  return { read: () => Promise.reject(error) };
}

function failingProposalReader(error: Error): PolledReader<readonly Proposal[]> {
  return { read: () => Promise.reject(error) };
}

describe("useAppState", () => {
  it("starts loading with default caps", () => {
    const focuses: Focus[] = [];
    const proposals: Proposal[] = [];
    const { result } = renderHook(() =>
      useAppState({
        focusReader: createFixtureFocusReader(focuses),
        proposalReader: createFixtureProposalReader(proposals),
        capsReader: createFixtureCapsReader(fixtureCaps),
      }),
    );
    expect(result.current.status).toBe("loading");
    expect(result.current.caps).toEqual(DEFAULT_CAPS);
  });

  it("becomes ready when both focuses and proposals resolve", async () => {
    const focuses: Focus[] = [{ id: "a", title: "A", description: "", created_at: "", tasks: [] }];
    const proposals: Proposal[] = [];
    const { result } = renderHook(() =>
      useAppState({
        focusReader: createFixtureFocusReader(focuses),
        proposalReader: createFixtureProposalReader(proposals),
        capsReader: createFixtureCapsReader(fixtureCaps),
      }),
    );
    await waitFor(() => {
      expect(result.current.status).toBe("ready");
    });
    if (result.current.status === "ready") {
      expect(result.current.focuses).toHaveLength(1);
      expect(result.current.proposals).toHaveLength(0);
    }
    await waitFor(() => {
      expect(result.current.caps).toEqual(fixtureCaps);
    });
  });

  it("surfaces a focus reader error", async () => {
    const err = new Error("boom-focus");
    const { result } = renderHook(() =>
      useAppState({
        focusReader: failingFocusReader(err),
        proposalReader: createFixtureProposalReader([]),
        capsReader: createFixtureCapsReader(fixtureCaps),
      }),
    );
    await waitFor(() => {
      expect(result.current.status).toBe("error");
    });
    if (result.current.status === "error") {
      expect(result.current.error.message).toBe("boom-focus");
    }
  });

  it("surfaces a proposal reader error", async () => {
    const err = new Error("boom-proposal");
    const { result } = renderHook(() =>
      useAppState({
        focusReader: createFixtureFocusReader([]),
        proposalReader: failingProposalReader(err),
        capsReader: createFixtureCapsReader(fixtureCaps),
      }),
    );
    await waitFor(() => {
      expect(result.current.status).toBe("error");
    });
    if (result.current.status === "error") {
      expect(result.current.error.message).toBe("boom-proposal");
    }
  });
});
