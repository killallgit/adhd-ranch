import React from "react";
import ReactDOM from "react-dom/client";
import { SettingsWindow } from "./components/SettingsWindow";
import { useSettingsWindow } from "./hooks/useSettingsWindow";
import "./styles.css";

function SettingsApp() {
  const {
    settings,
    monitors,
    notificationSources,
    debugOverlay,
    devtoolsOpen,
    update,
    setDebugOverlayEnabled,
    toggleDevtoolsOpen,
  } = useSettingsWindow();

  return (
    <SettingsWindow
      settings={settings}
      monitors={monitors}
      notificationSources={notificationSources}
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
