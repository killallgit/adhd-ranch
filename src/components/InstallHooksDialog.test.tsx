import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  ADD_MARKETPLACE_COMMAND,
  INSTALL_HOOKS_COMMAND,
  InstallHooksDialog,
} from "./InstallHooksDialog";

describe("InstallHooksDialog", () => {
  it("shows fresh-shell marketplace and install commands and copies both", async () => {
    const copyCommand = vi.fn().mockResolvedValue(undefined);
    render(<InstallHooksDialog copyCommand={copyCommand} install={vi.fn()} close={() => {}} />);

    expect(ADD_MARKETPLACE_COMMAND).toBe("claude plugin marketplace add killallgit/adhd-ranch");
    expect(INSTALL_HOOKS_COMMAND).toBe(
      "claude plugin install adhd-ranch-hooks@adhd-ranch --scope user",
    );
    expect(screen.getByText(/fresh.*shell/i)).toBeTruthy();
    expect(screen.getByText(ADD_MARKETPLACE_COMMAND)).toBeTruthy();
    expect(screen.getByText(INSTALL_HOOKS_COMMAND)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Copy commands" }));
    await waitFor(() =>
      expect(copyCommand).toHaveBeenCalledWith(
        `${ADD_MARKETPLACE_COMMAND}\n${INSTALL_HOOKS_COMMAND}`,
      ),
    );
    expect(await screen.findByText("Commands copied.")).toBeTruthy();
  });

  it("leaves the command readable when clipboard access fails", async () => {
    const copyCommand = vi.fn().mockRejectedValue(new Error("clipboard unavailable"));
    render(<InstallHooksDialog copyCommand={copyCommand} install={vi.fn()} close={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "Copy commands" }));
    expect(await screen.findByText(/Select the commands above/)).toBeTruthy();
    expect(screen.getByText(ADD_MARKETPLACE_COMMAND)).toBeTruthy();
    expect(screen.getByText(INSTALL_HOOKS_COMMAND)).toBeTruthy();
  });

  it("Done requests the host window to close", () => {
    const close = vi.fn();
    render(<InstallHooksDialog copyCommand={vi.fn()} install={vi.fn()} close={close} />);

    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(close).toHaveBeenCalledOnce();
  });

  it("installs hooks for the user when Install is clicked", async () => {
    const install = vi.fn().mockResolvedValue(undefined);
    render(<InstallHooksDialog copyCommand={vi.fn()} install={install} close={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Install hooks" }));
    await waitFor(() => expect(install).toHaveBeenCalledOnce());
    expect(await screen.findByText(/Hooks are installed/)).toBeTruthy();
  });

  it("reports installation failure and allows another attempt", async () => {
    const install = vi
      .fn()
      .mockRejectedValueOnce(new Error("Claude CLI unavailable"))
      .mockResolvedValueOnce(undefined);
    render(<InstallHooksDialog copyCommand={vi.fn()} install={install} close={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Install hooks" }));
    expect(await screen.findByText(/Claude CLI unavailable/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Install hooks" }).hasAttribute("disabled")).toBe(
      false,
    );

    fireEvent.click(screen.getByRole("button", { name: "Install hooks" }));
    await waitFor(() => expect(install).toHaveBeenCalledTimes(2));
    expect(await screen.findByText(/Hooks are installed/)).toBeTruthy();
  });

  it("prevents duplicate install clicks while Claude is working", async () => {
    let finishInstall: () => void = () => {};
    const install = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          finishInstall = resolve;
        }),
    );
    render(<InstallHooksDialog copyCommand={vi.fn()} install={install} close={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Install hooks" }));
    expect(screen.getByRole("button", { name: "Installing…" }).hasAttribute("disabled")).toBe(true);
    expect(install).toHaveBeenCalledOnce();

    finishInstall();
    expect(await screen.findByText(/Hooks are installed/)).toBeTruthy();
  });

  it("grants the install-hooks window permission to close", () => {
    const capability = JSON.parse(
      readFileSync(resolve(process.cwd(), "src-tauri/capabilities/install-hooks.json"), "utf8"),
    );
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), "src-tauri/tauri.conf.json"), "utf8"),
    );

    expect(capability.windows).toContain("install-hooks");
    expect(capability.permissions).toContain("core:window:allow-close");
    expect(config.app.security.capabilities ?? ["install-hooks"]).toContain("install-hooks");
  });
});
