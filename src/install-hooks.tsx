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
      close={() => getCurrentWindow().close().catch(console.error)}
    />
  </React.StrictMode>,
);
