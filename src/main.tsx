import React from "react";
import ReactDOM from "react-dom/client";
import { createTauriFocusReader } from "./api/tauriFocusReader";
import { createTauriProposalReader, createTauriProposalWriter } from "./api/tauriProposalReader";
import { App } from "./components/App";
import "./styles.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

const focusReader = createTauriFocusReader();
const proposalReader = createTauriProposalReader();
const proposalWriter = createTauriProposalWriter();

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <App
      focusReader={focusReader}
      proposalReader={proposalReader}
      proposalWriter={proposalWriter}
    />
  </React.StrictMode>,
);
