import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { INSTALL_HOOKS_COMMAND, InstallHooksDialog } from "./InstallHooksDialog";

describe("InstallHooksDialog", () => {
  it("shows the one-command Claude marketplace flow and copies it", async () => {
    const copyCommand = vi.fn().mockResolvedValue(undefined);
    render(<InstallHooksDialog copyCommand={copyCommand} close={() => {}} />);

    expect(screen.getByText(INSTALL_HOOKS_COMMAND)).toBeTruthy();
    expect(screen.getByText("User")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Copy command" }));
    await waitFor(() => expect(copyCommand).toHaveBeenCalledWith(INSTALL_HOOKS_COMMAND));
    expect(await screen.findByText("Command copied.")).toBeTruthy();
  });

  it("leaves the command readable when clipboard access fails", async () => {
    const copyCommand = vi.fn().mockRejectedValue(new Error("clipboard unavailable"));
    render(<InstallHooksDialog copyCommand={copyCommand} close={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "Copy command" }));
    expect(await screen.findByText(/Select the command above/)).toBeTruthy();
    expect(screen.getByText(INSTALL_HOOKS_COMMAND)).toBeTruthy();
  });
});
