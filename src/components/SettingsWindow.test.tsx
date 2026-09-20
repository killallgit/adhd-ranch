import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createRef } from "react";
import { describe, expect, it, vi } from "vitest";
import type { Settings } from "../types/settings";
import { SettingsWindow } from "./SettingsWindow";

const SETTINGS: Settings = {
  caps: { max_focuses: 5, max_tasks_per_focus: 7 },
  notifications: { sources: {} },
  widget: { always_on_top: false, confirm_delete: true },
  displays: { enabled_indices: [0] },
  agents: { enabled: false },
  pens: { max_size: 320 },
};

function renderSettings(onUpdate: (next: Settings) => void) {
  render(
    <SettingsWindow
      settings={SETTINGS}
      monitors={[]}
      debugOverlay={false}
      devtoolsOpen={false}
      onUpdate={onUpdate}
      onSetDebugOverlay={() => {}}
      onToggleDevtools={() => {}}
      containerRef={createRef<HTMLDivElement>()}
    />,
  );
}

describe("SettingsWindow number rows", () => {
  it("keeps a half-typed pen size on screen instead of discarding it", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max pen size") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "4");

    expect(input.value).toBe("4");
    expect(onUpdate).not.toHaveBeenCalled();
  });

  it("commits a pen size typed one digit at a time", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max pen size") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "480");

    expect(onUpdate).toHaveBeenLastCalledWith(expect.objectContaining({ pens: { max_size: 480 } }));
  });

  it("corrects a pen size below the floor when the field is left", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max pen size") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "4");
    await userEvent.tab();

    expect(onUpdate).toHaveBeenLastCalledWith(expect.objectContaining({ pens: { max_size: 160 } }));
  });

  it("leaves a row whose floor is a single digit working as before", async () => {
    const onUpdate = vi.fn();
    renderSettings(onUpdate);
    const input = screen.getByLabelText("Max focuses") as HTMLInputElement;

    await userEvent.clear(input);
    await userEvent.type(input, "3");

    expect(onUpdate).toHaveBeenLastCalledWith(
      expect.objectContaining({ caps: { max_focuses: 3, max_tasks_per_focus: 7 } }),
    );
  });
});
