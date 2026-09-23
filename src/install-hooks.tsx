import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import React from "react";
import ReactDOM from "react-dom/client";
import { InstallHooksDialog } from "./components/InstallHooksDialog";
import "./styles.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <InstallHooksDialog
      copyCommand={(command) => navigator.clipboard.writeText(command)}
      install={() => invoke("install_claude_hooks")}
      close={() => getCurrentWindow().close().catch(console.error)}
    />
  </React.StrictMode>,
);
