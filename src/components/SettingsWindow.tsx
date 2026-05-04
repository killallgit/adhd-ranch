import type React from "react";
import type { MonitorInfo } from "../types/monitor";
import type { Settings } from "../types/settings";

interface SettingsWindowProps {
  readonly settings: Settings | null;
  readonly monitors: MonitorInfo[];
  readonly debugOverlay: boolean;
  readonly devtoolsOpen: boolean;
  readonly onUpdate: (next: Settings) => void;
  readonly onSetDebugOverlay: (enabled: boolean) => void;
  readonly onToggleDevtools: () => void;
  readonly containerRef: React.RefObject<HTMLDivElement | null>;
}

interface ToggleRowProps {
  readonly label: string;
  readonly checked: boolean;
  readonly onChange: (v: boolean) => void;
}

function ToggleRow({ label, checked, onChange }: ToggleRowProps) {
  return (
    <label className="settings-row">
      <span className="settings-row-label">{label}</span>
      <input
        type="checkbox"
        className="settings-toggle"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
    </label>
  );
}

interface NumberRowProps {
  readonly label: string;
  readonly value: number;
  readonly min: number;
  readonly max: number;
  readonly onChange: (v: number) => void;
}

function NumberRow({ label, value, min, max, onChange }: NumberRowProps) {
  return (
    <label className="settings-row">
      <span className="settings-row-label">{label}</span>
      <input
        type="number"
        className="settings-number"
        min={min}
        max={max}
        value={value}
        onChange={(e) => {
          const v = Number.parseInt(e.target.value, 10);
          if (!Number.isNaN(v) && v >= min && v <= max) onChange(v);
        }}
      />
    </label>
  );
}

function toggleDisplayIndex(indices: readonly number[], idx: number): number[] {
  const next = indices.includes(idx)
    ? indices.filter((i) => i !== idx)
    : [...indices, idx].sort((a, b) => a - b);
  return next.length > 0 ? next : [idx];
}

export function SettingsWindow({
  settings,
  monitors,
  debugOverlay,
  devtoolsOpen,
  onUpdate,
  onSetDebugOverlay,
  onToggleDevtools,
  containerRef,
}: SettingsWindowProps) {
  return (
    <div ref={containerRef as React.Ref<HTMLDivElement>} className="settings-window">
      {settings && (
        <>
          <section className="settings-section">
            <h2 className="settings-section-title">General</h2>
            <NumberRow
              label="Max focuses"
              value={settings.caps.max_focuses}
              min={1}
              max={10}
              onChange={(v) =>
                onUpdate({ ...settings, caps: { ...settings.caps, max_focuses: v } })
              }
            />
            <NumberRow
              label="Max tasks per focus"
              value={settings.caps.max_tasks_per_focus}
              min={1}
              max={20}
              onChange={(v) =>
                onUpdate({ ...settings, caps: { ...settings.caps, max_tasks_per_focus: v } })
              }
            />
          </section>

          <section className="settings-section">
            <h2 className="settings-section-title">Widget</h2>
            <ToggleRow
              label="Always on Top"
              checked={settings.widget.always_on_top}
              onChange={(v) =>
                onUpdate({ ...settings, widget: { ...settings.widget, always_on_top: v } })
              }
            />
            <ToggleRow
              label="Confirm Before Delete"
              checked={settings.widget.confirm_delete}
              onChange={(v) =>
                onUpdate({ ...settings, widget: { ...settings.widget, confirm_delete: v } })
              }
            />
          </section>

          {monitors.length > 0 && (
            <section className="settings-section">
              <h2 className="settings-section-title">Displays</h2>
              {monitors.map((m) => (
                <ToggleRow
                  key={m.idx}
                  label={m.label}
                  checked={settings.displays.enabled_indices.includes(m.idx)}
                  onChange={() =>
                    onUpdate({
                      ...settings,
                      displays: {
                        enabled_indices: toggleDisplayIndex(
                          settings.displays.enabled_indices,
                          m.idx,
                        ),
                      },
                    })
                  }
                />
              ))}
            </section>
          )}

          <section className="settings-section">
            <h2 className="settings-section-title">Alerts</h2>
            <ToggleRow
              label="System Notifications"
              checked={settings.alerts.system_notifications}
              onChange={(v) =>
                onUpdate({
                  ...settings,
                  alerts: { ...settings.alerts, system_notifications: v },
                })
              }
            />
          </section>
        </>
      )}

      <section className="settings-section">
        <h2 className="settings-section-title">Debug</h2>
        <ToggleRow label="Debug Overlay" checked={debugOverlay} onChange={onSetDebugOverlay} />
        {import.meta.env.DEV && (
          <ToggleRow label="DevTools" checked={devtoolsOpen} onChange={onToggleDevtools} />
        )}
      </section>
    </div>
  );
}
