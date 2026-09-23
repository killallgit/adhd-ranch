import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { PolledState } from "../hooks/usePolledReader";
import type { AgentSession } from "../types/agentSession";
import type { HookFiring } from "../types/generated/HookFiring";
import type { HookWiring } from "../types/generated/HookWiring";
import { AgentDebugWindow } from "./AgentDebugWindow";

const WIRING: HookWiring = {
  agent: "Claude Code",
  settings_file: "/home/settings.json",
  client_bin: "/app/adhd-ranch-hook",
  socket_path: "/home/agent-hooks.sock",
  installed: true,
  enabled: true,
};

function ready<T>(value: T): PolledState<T> {
  return { status: "ready", value };
}

const failed: PolledState<never> = { status: "error", error: new Error("ipc is down") };
const loading: PolledState<never> = { status: "loading" };

const NO_SESSIONS = ready<readonly AgentSession[]>([]);
const NO_FIRINGS = ready<readonly HookFiring[]>([]);

describe("AgentDebugWindow", () => {
  it("says the ranch is idle when the reads succeed with nothing in them", () => {
    render(<AgentDebugWindow wiring={ready(WIRING)} sessions={NO_SESSIONS} firings={NO_FIRINGS} />);

    expect(screen.getByText(/No session has reported in/)).toBeTruthy();
    expect(screen.getByText(/Nothing has arrived/)).toBeTruthy();
  });

  it("reports a failed read instead of drawing it as an idle ranch", () => {
    render(<AgentDebugWindow wiring={ready(WIRING)} sessions={failed} firings={failed} />);

    expect(screen.queryByText(/No session has reported in/)).toBeNull();
    expect(screen.queryByText(/Nothing has arrived/)).toBeNull();
    expect(screen.getAllByText(/Could not read/).length).toBe(2);
  });

  it("does not claim the hooks are missing while the wiring is still being read", () => {
    render(<AgentDebugWindow wiring={loading} sessions={NO_SESSIONS} firings={NO_FIRINGS} />);

    expect(screen.queryByText(/hooks verified/)).toBeNull();
  });

  it("shows a failed wiring read rather than hiding the section", () => {
    render(<AgentDebugWindow wiring={failed} sessions={NO_SESSIONS} firings={NO_FIRINGS} />);

    expect(screen.getByText(/Could not read the wiring/)).toBeTruthy();
  });
});
