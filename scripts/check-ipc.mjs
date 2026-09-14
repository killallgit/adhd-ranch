import {
  findIpcDrift,
  parseEmittedEvents,
  parseInvokedCommands,
  parseListenedEvents,
  parseRegisteredCommands,
} from "./unused/ipc.mjs";
import { FRONTEND_SOURCES, RUST_SOURCES, readSources } from "./unused/sources.mjs";

const rust = readSources(RUST_SOURCES).join("\n");
const frontend = readSources(FRONTEND_SOURCES).join("\n");

const drift = findIpcDrift({
  registered: parseRegisteredCommands(rust),
  invoked: parseInvokedCommands(frontend),
  emitted: parseEmittedEvents(rust),
  listened: parseListenedEvents(frontend),
});

const labels = {
  unusedCommands: "Tauri commands registered but never invoked by the frontend",
  unregisteredCommands: "Commands invoked by the frontend but not registered in generate_handler!",
  unlistenedEvents: "Events emitted by Rust but never listened to by the frontend",
  unemittedEvents: "Events listened to by the frontend but never emitted by Rust",
};

const problems = Object.entries(drift).filter(([, names]) => names.length > 0);

for (const [key, names] of problems) {
  console.error(`${labels[key]}:`);
  for (const name of names) console.error(`  - ${name}`);
}

if (problems.length > 0) process.exit(1);
console.log("IPC boundary: no unused commands or events");
