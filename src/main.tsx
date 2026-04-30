import React from "react";
import ReactDOM from "react-dom/client";
import { createTauriFocusReader } from "./api/tauriFocusReader";
import { App } from "./components/App";
import "./styles.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

const focusReader = createTauriFocusReader();

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <App focusReader={focusReader} />
  </React.StrictMode>,
);
