import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";

const { invoke, close } = vi.hoisted(() => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  close: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ close }),
}));

beforeEach(() => {
  document.body.innerHTML = '<div id="root"></div>';
  invoke.mockClear();
  close.mockClear();
});

it("routes Install and Done to the Tauri host", async () => {
  await act(async () => {
    await import("./install-hooks");
  });

  fireEvent.click(await screen.findByRole("button", { name: "Install hooks" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("install_claude_hooks"));

  fireEvent.click(screen.getByRole("button", { name: "Done" }));
  await waitFor(() => expect(close).toHaveBeenCalledOnce());
});
