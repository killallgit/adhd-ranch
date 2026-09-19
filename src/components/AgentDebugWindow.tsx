import { clockTime, newestFirst, shortId } from "../lib/session/firings";
import type { AgentSession } from "../types/agentSession";
import type { HookFiring } from "../types/generated/HookFiring";
import type { HookWiring } from "../types/generated/HookWiring";

interface AgentDebugWindowProps {
  readonly wiring: HookWiring | null;
  readonly sessions: readonly AgentSession[];
  readonly firings: readonly HookFiring[];
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

function Wiring({ wiring }: { readonly wiring: HookWiring }) {
  return (
    <section className="settings-section">
      <h2 className="settings-section-title">{wiring.agent}</h2>
      <div className="agent-debug-lights">
        <Light on={wiring.enabled} label="agents enabled" />
        <Light on={wiring.installed} label="hooks installed" />
      </div>
      <PathRow label="socket" value={wiring.socket_path} />
      <PathRow label="client" value={wiring.client_bin} />
      <PathRow label="settings" value={wiring.settings_file} />
    </section>
  );
}

function Sessions({ sessions }: { readonly sessions: readonly AgentSession[] }) {
  return (
    <section className="settings-section">
      <h2 className="settings-section-title">Sessions ({sessions.length})</h2>
      {sessions.length === 0 ? (
        <p className="agent-debug-empty">No session has reported in.</p>
      ) : (
        sessions.map((session) => (
          <div className="agent-debug-row" key={session.id}>
            <span className="agent-debug-when">{shortId(session.id)}</span>
            <span className="agent-debug-action">{session.activity}</span>
            <span className="agent-debug-pen">{session.pen.name}</span>
          </div>
        ))
      )}
    </section>
  );
}

function Firings({ firings }: { readonly firings: readonly HookFiring[] }) {
  return (
    <section className="settings-section">
      <h2 className="settings-section-title">Firings ({firings.length})</h2>
      {firings.length === 0 ? (
        <p className="agent-debug-empty">
          Nothing has arrived. Submit a prompt in an agent and watch this line.
        </p>
      ) : (
        <div className="agent-debug-feed">
          {newestFirst(firings).map((firing, index) => (
            <div
              className="agent-debug-row"
              // Firings are immutable and only ever prepended, so position in the
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
      {wiring ? <Wiring wiring={wiring} /> : null}
      <Sessions sessions={sessions} />
      <Firings firings={firings} />
    </div>
  );
}
