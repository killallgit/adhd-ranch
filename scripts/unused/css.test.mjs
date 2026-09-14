import { describe, expect, it } from "vitest";
import { findUnusedClasses, parseClassSelectors } from "./css.mjs";

describe("parseClassSelectors", () => {
  it("reads class names from compound and pseudo selectors", () => {
    const css = ".card:hover, .card .title > .badge--red { color: red; }";

    expect(parseClassSelectors(css)).toEqual(["badge--red", "card", "title"]);
  });

  it("ignores dotted values inside declarations", () => {
    const css = ".card { background: url(pig.png); margin: .5em; }";

    expect(parseClassSelectors(css)).toEqual(["card"]);
  });

  it("ignores commented-out rules", () => {
    const css = "/* .old { color: red; } */ .card { color: blue; }";

    expect(parseClassSelectors(css)).toEqual(["card"]);
  });

  it("reads through nested at-rule blocks", () => {
    const css =
      "@keyframes pulse { 0% { opacity: 1; } } @media (width > 1px) { .card { color: red; } }";

    expect(parseClassSelectors(css)).toEqual(["card"]);
  });
});

describe("findUnusedClasses", () => {
  it("treats a class referenced in a template literal as used", () => {
    const css = ".pig-sprite--expired { opacity: 0.5; }";
    const sources = ["className={`pig-sprite${expired ? ' pig-sprite--expired' : ''}`}"];

    expect(findUnusedClasses(css, sources)).toEqual([]);
  });

  it("reports classes no source mentions", () => {
    const css = ".focus-card { color: red; } .pig-sprite { color: blue; }";
    const sources = ['<div className="pig-sprite" />'];

    expect(findUnusedClasses(css, sources)).toEqual(["focus-card"]);
  });

  it("does not treat a longer class name as a use of its prefix", () => {
    const css = ".task { color: red; }";
    const sources = ['<li className="task-row" />'];

    expect(findUnusedClasses(css, sources)).toEqual(["task"]);
  });
});
