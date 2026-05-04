import { useCallback, useEffect, useState } from "react";
import { getDebugOverlay, getDevtoolsOpen, setDebugOverlay, toggleDevtools } from "../api/debug";
import { getMonitors } from "../api/monitors";
import { getSettings, updateSettings } from "../api/settings";
import type { MonitorInfo } from "../types/monitor";
import type { Settings } from "../types/settings";

export interface SettingsWindowState {
  settings: Settings | null;
  monitors: MonitorInfo[];
  debugOverlay: boolean;
  devtoolsOpen: boolean;
  update: (next: Settings) => void;
  setDebugOverlayEnabled: (enabled: boolean) => void;
  toggleDevtoolsOpen: () => void;
}

export function useSettingsWindow(): SettingsWindowState {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [debugOverlay, setDebugOverlayState] = useState(false); // real state loaded from backend on mount
  const [devtoolsOpen, setDevtoolsOpen] = useState(false);

  useEffect(() => {
    getSettings().then(setSettings).catch(console.error);
    getMonitors().then(setMonitors).catch(console.error);
    getDebugOverlay().then(setDebugOverlayState).catch(console.error);
    getDevtoolsOpen().then(setDevtoolsOpen).catch(console.error);
  }, []);

  const update = useCallback((next: Settings) => {
    setSettings(next);
    updateSettings(next).catch(console.error);
  }, []);

  const setDebugOverlayEnabled = useCallback((enabled: boolean) => {
    setDebugOverlayState(enabled);
    setDebugOverlay(enabled).catch(console.error);
  }, []);

  const toggleDevtoolsOpen = useCallback(() => {
    setDevtoolsOpen((prev) => !prev);
    toggleDevtools().catch(console.error);
  }, []);

  return {
    settings,
    monitors,
    debugOverlay,
    devtoolsOpen,
    update,
    setDebugOverlayEnabled,
    toggleDevtoolsOpen,
  };
}
