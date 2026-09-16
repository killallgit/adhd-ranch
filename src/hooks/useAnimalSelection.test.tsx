import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Animal } from "../types/animal";
import { useAnimalSelection } from "./useAnimalSelection";

vi.mock("../api/pig", () => ({
  subscribeOpenFocusDetail: vi.fn().mockResolvedValue(() => {}),
}));

const focusAnimal = (id: string): Animal => ({
  kind: "focus",
  id,
  name: id,
  expired: false,
  scale: 1,
  focus: { id, title: id, description: "", created_at: "", tasks: [] },
});

const agentAnimal = (id: string): Animal => ({
  kind: "agent",
  id: `agent:${id}`,
  name: id,
  expired: false,
  scale: 1,
  session: { id, name: id },
});

describe("useAnimalSelection", () => {
  it("selects a Focus Animal", () => {
    const { result } = renderHook(() => useAnimalSelection([focusAnimal("a")]));

    act(() => result.current.select("a"));

    expect(result.current.selected?.focus.id).toBe("a");
  });

  it("does not select an agent Animal", () => {
    const { result } = renderHook(() => useAnimalSelection([agentAnimal("session-1")]));

    act(() => result.current.select("agent:session-1"));

    expect(result.current.selected).toBeNull();
  });

  it("resolves to none when the selected Animal is gone", () => {
    const { result, rerender } = renderHook(({ animals }) => useAnimalSelection(animals), {
      initialProps: { animals: [focusAnimal("a")] as readonly Animal[] },
    });
    act(() => result.current.select("a"));

    rerender({ animals: [] });

    expect(result.current.selected).toBeNull();
  });

  it("closes the selection", () => {
    const { result } = renderHook(() => useAnimalSelection([focusAnimal("a")]));
    act(() => result.current.select("a"));

    act(() => result.current.close());

    expect(result.current.selected).toBeNull();
  });
});
