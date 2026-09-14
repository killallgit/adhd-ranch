import { globSync, readFileSync } from "node:fs";

const isExcluded = (path) => path.includes(".test.") || path.includes("src/types/generated");

export function readSources(patterns) {
  return globSync(patterns, { exclude: isExcluded })
    .sort()
    .map((path) => readFileSync(path, "utf8"));
}

export const RUST_SOURCES = ["src-tauri/src/**/*.rs", "crates/*/src/**/*.rs"];
export const FRONTEND_SOURCES = ["src/**/*.ts", "src/**/*.tsx"];
export const HTML_ENTRIES = ["*.html"];
export const STYLESHEETS = ["src/**/*.css"];
