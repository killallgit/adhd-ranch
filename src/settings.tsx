import { LogicalSize, getCurrentWindow } from "@tauri-apps/api/window";
import React, { useEffect, useRef } from "react";
import ReactDOM from "react-dom/client";
import { SettingsWindow } from "./components/SettingsWindow";
import { useSettingsWindow } from "./hooks/useSettingsWindow";
import "./styles.css";

function SettingsApp() {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const {
    settings,
    monitors,
    debugOverlay,
    devtoolsOpen,
    update,
    setDebugOverlayEnabled,
    toggleDevtoolsOpen,
  } = useSettingsWindow();

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const win = getCurrentWindow();
    const observer = new ResizeObserver(() => {
      const h = Math.ceil(el.getBoundingClientRect().height);
      win.setSize(new LogicalSize(380, h)).catch(console.error);
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return (
    <SettingsWindow
      containerRef={containerRef}
      settings={settings}
      monitors={monitors}
      debugOverlay={debugOverlay}
      devtoolsOpen={devtoolsOpen}
      onUpdate={update}
      onSetDebugOverlay={setDebugOverlayEnabled}
      onToggleDevtools={toggleDevtoolsOpen}
    />
  );
}

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <SettingsApp />
  </React.StrictMode>,
);
