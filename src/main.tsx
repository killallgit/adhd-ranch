import React from "react";
import ReactDOM from "react-dom/client";
import { type WriteOutcome, createTauriFocusWriter } from "./api/focusWriter";
import { createTauriFocusReader } from "./api/tauriFocusReader";
import { App } from "./components/App";
import "./styles.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

const focusReader = createTauriFocusReader();
const focusWriter = createTauriFocusWriter();

function reportWriteFailure(op: string, outcome: WriteOutcome) {
  if (!outcome.ok) console.warn(`[adhd-ranch] ${op} failed`, outcome.kind, outcome.message);
}

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <App focusReader={focusReader} focusWriter={focusWriter} onWriteFailure={reportWriteFailure} />
  </React.StrictMode>,
);
