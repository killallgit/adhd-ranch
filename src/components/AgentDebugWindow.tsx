import type { PolledState } from "../hooks/usePolledReader";
import { clockTime, newestFirst, shortId } from "../lib/session/firings";
import type { AgentSession } from "../types/agentSession";
import type { HookFiring } from "../types/generated/HookFiring";
import type { HookWiring } from "../types/generated/HookWiring";

interface AgentDebugWindowProps {
  readonly wiring: PolledState<HookWiring>;
  readonly sessions: PolledState<readonly AgentSession[]>;
  readonly firings: PolledState<readonly HookFiring[]>;
}

// A read that failed must never be drawn as a read that came back empty. This window
// exists to tell silence apart from breakage, and an empty list in place of an error
// is the same lie one layer up.
function NotReady<T>({ state, what }: { readonly state: PolledState<T>; readonly what: string }) {
  if (state.status === "loading") return <p className="agent-debug-empty">Reading {what}…</p>;
  if (state.status === "error") {
    return (
      <p className="agent-debug-failed">
        Could not read {what} — {state.error.message}
      </p>
    );
  }
  return null;
}

function ready<T>(state: PolledState<T>): T | null {
  return state.status === "ready" ? state.value : null;
}

function Light({ on, label }: { readonly on: boolean; readonly label: string }) {
  return (
    <span className="agent-debug-light">
      <span className={on ? "agent-debug-dot-on" : "agent-debug-dot-off"} />
      {label}
    </span>
  );
}

function PathRow({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div className="agent-debug-path">
      <span className="agent-debug-path-label">{label}</span>
      <span className="agent-debug-path-value" title={value}>
        {value}
      </span>
    </div>
  );
}

function Wiring({ state }: { readonly state: PolledState<HookWiring> }) {
  const wiring = ready(state);
  return (
    <section className="settings-section">
      <h2 className="settings-section-title">{wiring?.agent ?? "Agent"}</h2>
      <NotReady state={state} what="the wiring" />
      {wiring && (
        <>
          <div className="agent-debug-lights">
            <Light on={wiring.enabled} label="agents enabled" />
            <Light on={wiring.installed} label="hooks installed" />
          </div>
          <PathRow label="socket" value={wiring.socket_path} />
          <PathRow label="client" value={wiring.client_bin} />
          <PathRow label="settings" value={wiring.settings_file} />
        </>
      )}
    </section>
  );
}

function Sessions({ state }: { readonly state: PolledState<readonly AgentSession[]> }) {
  const sessions = ready(state);
  return (
    <section className="settings-section">
      <h2 className="settings-section-title">Sessions{sessions ? ` (${sessions.length})` : ""}</h2>
      <NotReady state={state} what="sessions" />
      {sessions?.length === 0 && <p className="agent-debug-empty">No session has reported in.</p>}
      {sessions?.map((session) => (
        <div className="agent-debug-row" key={session.id}>
          <span className="agent-debug-when">{shortId(session.id)}</span>
          <span className="agent-debug-action">{session.activity}</span>
          <span className="agent-debug-pen">{session.pen.name}</span>
        </div>
      ))}
    </section>
  );
}

function Firings({ state }: { readonly state: PolledState<readonly HookFiring[]> }) {
  const firings = ready(state);
  return (
    <section className="settings-section">
      <h2 className="settings-section-title">Firings{firings ? ` (${firings.length})` : ""}</h2>
      <NotReady state={state} what="firings" />
      {firings?.length === 0 && (
        <p className="agent-debug-empty">
          Nothing has arrived. Submit a prompt in an agent and watch this line.
        </p>
      )}
      {firings && firings.length > 0 && (
        <div className="agent-debug-feed">
          {newestFirst(firings).map((firing, index) => (
            <div
              className="agent-debug-row"
              // Firings are immutable and only ever appended, so position in the
              // reversed list is stable for as long as a row is on screen.
              key={`${firing.at}-${index}`}
            >
              <span className="agent-debug-when">{clockTime(firing.at)}</span>
              <span className="agent-debug-action">{firing.action}</span>
              <span className="agent-debug-pen">{shortId(firing.session_id)}</span>
              <span className={firing.changed ? "agent-debug-kept" : "agent-debug-ignored"}>
                {firing.changed ? "applied" : "no change"}
              </span>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

export function AgentDebugWindow({ wiring, sessions, firings }: AgentDebugWindowProps) {
  return (
    <div className="settings-window agent-debug-window">
      <Wiring state={wiring} />
      <Sessions state={sessions} />
      <Firings state={firings} />
    </div>
  );
}
