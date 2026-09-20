import type React from "react";
import { useState } from "react";
import { PEN_SIZE_RANGE } from "../lib/session/pens";
import type { MonitorInfo } from "../types/monitor";
import type { Settings } from "../types/settings";

const NOTIFICATION_SOURCES = [
  { key: "timer_expired", label: "Timer expired" },
  { key: "task_timer_expired", label: "Task timer expired" },
  { key: "focuses_over_cap", label: "Too many focuses" },
  { key: "tasks_over_cap", label: "Too many tasks in a focus" },
] as const;

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
  // Committing only in-range values makes a field with a three-digit `min` impossible
  // to type into: "4" on the way to "480" is below 160, so it was discarded and the
  // field snapped back. The draft holds whatever is typed, and leaving the field
  // commits the nearest legal value — the same correction a hand-edited settings.yaml
  // gets from `PenConfig::clamped`.
  const [draft, setDraft] = useState<string | null>(null);

  return (
    <label className="settings-row">
      <span className="settings-row-label">{label}</span>
      <input
        type="number"
        className="settings-number"
        min={min}
        max={max}
        value={draft ?? value}
        onChange={(e) => {
          setDraft(e.target.value);
          const v = Number.parseInt(e.target.value, 10);
          if (!Number.isNaN(v) && v >= min && v <= max) onChange(v);
        }}
        onBlur={() => {
          if (draft === null) return;
          const v = Number.parseInt(draft, 10);
          setDraft(null);
          if (!Number.isNaN(v)) onChange(Math.min(Math.max(v, min), max));
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

          <section className="settings-section">
            <h2 className="settings-section-title">Pens</h2>
            <NumberRow
              label="Max pen size"
              value={settings.pens.max_size}
              min={PEN_SIZE_RANGE.min}
              max={PEN_SIZE_RANGE.max}
              onChange={(v) => onUpdate({ ...settings, pens: { max_size: v } })}
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
            <h2 className="settings-section-title">Notifications</h2>
            {NOTIFICATION_SOURCES.map(({ key, label }) => (
              <ToggleRow
                key={key}
                label={label}
                checked={settings.notifications.sources[key] ?? true}
                onChange={(v) =>
                  onUpdate({
                    ...settings,
                    notifications: {
                      sources: { ...settings.notifications.sources, [key]: v },
                    },
                  })
                }
              />
            ))}
          </section>
        </>
      )}

      <section className="settings-section">
        <h2 className="settings-section-title">Debug</h2>
        <ToggleRow label="Debug Overlay" checked={debugOverlay} onChange={onSetDebugOverlay} />
        {import.meta.env.DEV && (
          <ToggleRow label="DevTools" checked={devtoolsOpen} onChange={() => onToggleDevtools()} />
        )}
      </section>
    </div>
  );
}
