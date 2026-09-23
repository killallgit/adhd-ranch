import { useState } from "react";

export const INSTALL_HOOKS_COMMAND =
  "/plugin install adhd-ranch-hooks --marketplace killallgit/adhd-ranch";

interface InstallHooksDialogProps {
  readonly copyCommand: (command: string) => Promise<void>;
  readonly close: () => void;
}

export function InstallHooksDialog({ copyCommand, close }: InstallHooksDialogProps) {
  const [copyMessage, setCopyMessage] = useState("");

  async function copy() {
    try {
      await copyCommand(INSTALL_HOOKS_COMMAND);
      setCopyMessage("Command copied.");
    } catch {
      setCopyMessage("Copy unavailable. Select the command above to copy it manually.");
    }
  }

  return (
    <main className="install-hooks-dialog">
      <h1>Install Claude hooks</h1>
      <p>Run this in Claude Code to connect session activity to ADHD Ranch:</p>
      <div className="install-hooks-command">
        <code>{INSTALL_HOOKS_COMMAND}</code>
      </div>
      <div className="install-hooks-actions">
        <button type="button" className="install-hooks-copy" onClick={copy}>
          Copy command
        </button>
        <button type="button" className="install-hooks-done" onClick={close}>
          Done
        </button>
      </div>
      <output className="install-hooks-copy-message">{copyMessage}</output>
      <p className="install-hooks-note">
        Choose <strong>User</strong> scope so it works in every project. Follow Claude's reload
        prompt if this session needs to activate the plugin.
      </p>
    </main>
  );
}
