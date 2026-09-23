import { useState } from "react";

export const ADD_MARKETPLACE_COMMAND = "claude plugin marketplace add killallgit/adhd-ranch";
export const INSTALL_HOOKS_COMMAND =
  "claude plugin install adhd-ranch-hooks@adhd-ranch --scope user";

interface InstallHooksDialogProps {
  readonly copyCommand: (command: string) => Promise<void>;
  readonly install: () => Promise<void>;
  readonly close: () => void;
}

export function InstallHooksDialog({ copyCommand, install, close }: InstallHooksDialogProps) {
  const [copyMessage, setCopyMessage] = useState("");
  const [installState, setInstallState] = useState<"idle" | "installing" | "installed" | "error">(
    "idle",
  );
  const [installMessage, setInstallMessage] = useState("");

  async function copy() {
    try {
      await copyCommand(`${ADD_MARKETPLACE_COMMAND}\n${INSTALL_HOOKS_COMMAND}`);
      setCopyMessage("Commands copied.");
    } catch {
      setCopyMessage("Copy unavailable. Select the commands above to copy them manually.");
    }
  }

  async function runInstall() {
    setInstallState("installing");
    setInstallMessage("Installing hooks…");
    try {
      await install();
      setInstallState("installed");
      setInstallMessage("Hooks are installed for your user. Reload Claude Code to activate them.");
    } catch (error) {
      setInstallState("error");
      setInstallMessage(
        `Install failed: ${error instanceof Error ? error.message : String(error)}. You can use the shell commands below.`,
      );
    }
  }

  return (
    <main className="install-hooks-dialog">
      <h1>Install hooks</h1>
      <p>Connect Claude Code sessions to ADHD Ranch across all your projects.</p>
      <div className="install-hooks-actions">
        <button
          type="button"
          className="install-hooks-install"
          onClick={runInstall}
          disabled={installState === "installing" || installState === "installed"}
        >
          {installState === "installing" ? "Installing…" : "Install hooks"}
        </button>
        <button type="button" className="install-hooks-copy" onClick={copy}>
          Copy commands
        </button>
        <button type="button" className="install-hooks-done" onClick={close}>
          Done
        </button>
      </div>
      <output
        className="install-hooks-install-message"
        role={installState === "error" ? "alert" : "status"}
      >
        {installMessage}
      </output>
      <p>Or, in a fresh shell (not inside Claude Code), run these commands:</p>
      <ol className="install-hooks-steps">
        <li>
          Add the Ranch marketplace:
          <div className="install-hooks-command">
            <code>{ADD_MARKETPLACE_COMMAND}</code>
          </div>
        </li>
        <li>
          Install the hooks for your user:
          <div className="install-hooks-command">
            <code>{INSTALL_HOOKS_COMMAND}</code>
          </div>
        </li>
      </ol>
      <output className="install-hooks-copy-message">{copyMessage}</output>
      <p className="install-hooks-note">
        If the marketplace is already added, skip the first command. Restart Claude Code or run
        <code> /reload-plugins</code> in an existing session after installing.
      </p>
    </main>
  );
}
