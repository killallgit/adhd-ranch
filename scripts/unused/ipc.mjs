const HANDLER_LIST = /generate_handler!\s*\[([\s\S]*?)\]/;
// Rust events are only detectable when the name is a literal at the emit site or a `*_EVENT` const.
const EVENT_CONST = /const\s+[A-Z0-9_]+_EVENT\s*:\s*&str\s*=\s*"([^"]+)"/g;
const EMIT_LITERAL = /\.emit\(\s*"([^"]+)"/g;
const INVOKE_CALL = /\b(?:invoke|runInvoke)\b[^(\n]*\(\s*"([^"]+)"/g;
const INVOKE_KEY = /\binvokeKey\s*:\s*"([^"]+)"/g;
const LISTEN_CALL = /\blisten\b[^(\n]*\(\s*"([^"]+)"/g;
const EVENT_KEY = /\beventKey\s*:\s*"([^"]+)"/g;

const captures = (text, pattern) => [...text.matchAll(pattern)].map((match) => match[1]);

const uniqueSorted = (names) => [...new Set(names)].sort();

const difference = (left, right) => {
  const exclude = new Set(right);
  return uniqueSorted(left.filter((name) => !exclude.has(name)));
};

export function parseRegisteredCommands(rust) {
  const list = HANDLER_LIST.exec(rust);
  if (!list) return [];
  return uniqueSorted(
    list[1]
      .split(",")
      .map((entry) => entry.trim())
      .filter(Boolean)
      .map((entry) => entry.split("::").pop()),
  );
}

export function parseEmittedEvents(rust) {
  return uniqueSorted([...captures(rust, EVENT_CONST), ...captures(rust, EMIT_LITERAL)]);
}

export function parseInvokedCommands(ts) {
  return uniqueSorted([...captures(ts, INVOKE_CALL), ...captures(ts, INVOKE_KEY)]);
}

export function parseListenedEvents(ts) {
  return uniqueSorted([...captures(ts, LISTEN_CALL), ...captures(ts, EVENT_KEY)]);
}

export function findIpcDrift({ registered, invoked, emitted, listened }) {
  return {
    unusedCommands: difference(registered, invoked),
    unregisteredCommands: difference(invoked, registered),
    unlistenedEvents: difference(emitted, listened),
    unemittedEvents: difference(listened, emitted),
  };
}
