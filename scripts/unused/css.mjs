const COMMENT = /\/\*[\s\S]*?\*\//g;
const INNERMOST_BLOCK = /\{[^{}]*\}/g;
const CLASS_SELECTOR = /\.(-?[A-Za-z_][\w-]*)/g;
const TOKEN_SEPARATOR = /[^\w-]+/;

// Declaration blocks are stripped first so values like `url(a.png)` or `.5em` are not read as classes.
export function parseClassSelectors(css) {
  const selectors = css.replace(COMMENT, "").replace(INNERMOST_BLOCK, "{}");
  return [...new Set([...selectors.matchAll(CLASS_SELECTOR)].map((match) => match[1]))].sort();
}

// Whole-word matching keeps dynamic names like `pig-sprite--expired` in template literals visible.
function collectTokens(sources) {
  return new Set(sources.flatMap((source) => source.split(TOKEN_SEPARATOR)));
}

export function findUnusedClasses(css, sources) {
  const tokens = collectTokens(sources);
  return parseClassSelectors(css).filter((name) => !tokens.has(name));
}
