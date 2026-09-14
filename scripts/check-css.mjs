import { findUnusedClasses } from "./unused/css.mjs";
import { FRONTEND_SOURCES, HTML_ENTRIES, STYLESHEETS, readSources } from "./unused/sources.mjs";

const css = readSources(STYLESHEETS).join("\n");
const sources = readSources([...FRONTEND_SOURCES, ...HTML_ENTRIES]);
const unused = findUnusedClasses(css, sources);

if (unused.length > 0) {
  console.error("CSS classes not referenced by any source file:");
  for (const name of unused) console.error(`  - .${name}`);
  process.exit(1);
}
console.log("CSS: every class is referenced");
