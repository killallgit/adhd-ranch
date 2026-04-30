import React from "react";
import ReactDOM from "react-dom/client";
import { createFixtureFocusReader } from "./api/fixtureFocusReader";
import { App } from "./components/App";
import { HARDCODED_FOCUSES } from "./data/fixtures";
import "./styles.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

const focusReader = createFixtureFocusReader(HARDCODED_FOCUSES);

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <App focusReader={focusReader} />
  </React.StrictMode>,
);
