import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { type Caps, type CapsReader, DEFAULT_CAPS } from "../api/caps";
import { createFixtureFocusReader } from "../api/fixtureFocusReader";
import { createFixtureProposalReader } from "../api/fixtureProposalReader";
import type { FocusReader } from "../api/focuses";
import type { ProposalReader } from "../api/proposals";
import type { Focus } from "../types/focus";
import type { Proposal } from "../types/proposal";
import { useAppState } from "./useAppState";

const fixtureCaps: Caps = { max_focuses: 3, max_tasks_per_focus: 4 };

function fixtureCapsReader(caps: Caps = fixtureCaps): CapsReader {
  return { get: () => Promise.resolve(caps) };
}

function failingFocusReader(error: Error): FocusReader {
  return { list: () => Promise.reject(error) };
}

function failingProposalReader(error: Error): ProposalReader {
  return { list: () => Promise.reject(error) };
}

describe("useAppState", () => {
  it("starts loading with default caps", () => {
    const focuses: Focus[] = [];
    const proposals: Proposal[] = [];
    const { result } = renderHook(() =>
      useAppState({
        focusReader: createFixtureFocusReader(focuses),
        proposalReader: createFixtureProposalReader(proposals),
        capsReader: fixtureCapsReader(),
      }),
    );
    expect(result.current.status).toBe("loading");
    expect(result.current.caps).toEqual(DEFAULT_CAPS);
  });

  it("becomes ready when both focuses and proposals resolve", async () => {
    const focuses: Focus[] = [{ id: "a", title: "A", description: "", tasks: [] }];
    const proposals: Proposal[] = [];
    const { result } = renderHook(() =>
      useAppState({
        focusReader: createFixtureFocusReader(focuses),
        proposalReader: createFixtureProposalReader(proposals),
        capsReader: fixtureCapsReader(),
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
        capsReader: fixtureCapsReader(),
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
        capsReader: fixtureCapsReader(),
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
