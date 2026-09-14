import { describe, expect, it } from "vitest";
import {
  findIpcDrift,
  parseEmittedEvents,
  parseInvokedCommands,
  parseListenedEvents,
  parseRegisteredCommands,
} from "./ipc.mjs";

describe("parseRegisteredCommands", () => {
  it("returns the last path segment of each generate_handler entry", () => {
    const rust = `.invoke_handler(tauri::generate_handler![
            ui_bridge::list_focuses,
            ui_bridge::get_caps,
        ])`;

    expect(parseRegisteredCommands(rust)).toEqual(["get_caps", "list_focuses"]);
  });

  it("returns nothing when no handler list exists", () => {
    expect(parseRegisteredCommands("fn main() {}")).toEqual([]);
  });
});

describe("parseEmittedEvents", () => {
  it("reads literal emit names and *_EVENT consts", () => {
    const rust = `
      pub const TIMER_EXPIRED_EVENT: &str = "timer-expired";
      let _ = win.emit("gather-pigs", ());
      handle.emit(TIMER_EXPIRED_EVENT, payload);
    `;

    expect(parseEmittedEvents(rust)).toEqual(["gather-pigs", "timer-expired"]);
  });
});

describe("parseInvokedCommands", () => {
  it("reads invoke calls with generics, wrapper calls, and reader keys", () => {
    const ts = `
      invoke<boolean>("get_debug_overlay");
      runInvoke("create_focus", { title });
      createTauriReader({ invokeKey: "list_focuses" });
      invoke<unknown>(cmd, args);
    `;

    expect(parseInvokedCommands(ts)).toEqual(["create_focus", "get_debug_overlay", "list_focuses"]);
  });
});

describe("parseListenedEvents", () => {
  it("reads listen calls and reader event keys", () => {
    const ts = `
      listen<string>("open-focus-detail", cb);
      createTauriReader({ eventKey: "focuses-changed" });
    `;

    expect(parseListenedEvents(ts)).toEqual(["focuses-changed", "open-focus-detail"]);
  });
});

describe("findIpcDrift", () => {
  it("reports registered commands the frontend never invokes", () => {
    const drift = findIpcDrift({
      registered: ["get_caps", "list_focuses"],
      invoked: ["list_focuses"],
      emitted: [],
      listened: [],
    });

    expect(drift.unusedCommands).toEqual(["get_caps"]);
  });

  it("reports invoked commands missing from the handler list", () => {
    const drift = findIpcDrift({
      registered: [],
      invoked: ["list_focuses"],
      emitted: [],
      listened: [],
    });

    expect(drift.unregisteredCommands).toEqual(["list_focuses"]);
  });

  it("reports emitted events nobody listens to", () => {
    const drift = findIpcDrift({
      registered: [],
      invoked: [],
      emitted: ["proposals-changed", "focuses-changed"],
      listened: ["focuses-changed"],
    });

    expect(drift.unlistenedEvents).toEqual(["proposals-changed"]);
  });

  it("reports listened events Rust never emits", () => {
    const drift = findIpcDrift({
      registered: [],
      invoked: [],
      emitted: [],
      listened: ["gather-pigs"],
    });

    expect(drift.unemittedEvents).toEqual(["gather-pigs"]);
  });
});
