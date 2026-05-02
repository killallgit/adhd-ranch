import React from "react";
import ReactDOM from "react-dom/client";
import { NewFocusWindow } from "./components/NewFocusWindow";
import "./styles.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <NewFocusWindow />
  </React.StrictMode>,
);
