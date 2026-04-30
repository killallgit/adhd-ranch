# 003 — Markdown storage + file watcher (read path)

## Parent PRD

`PRD.md`

## What to build

Replace the hardcoded fixture with real Focuses read from disk. Implements the storage layer end-to-end: parse `~/.adhd-ranch/focuses/<slug>/focus.md` (YAML frontmatter + checkbox bullet body), watch the directory tree with the `notify` crate, and push updates into the React view via Tauri events.

See `PRD.md` FR1 (Focus storage) and `CONTEXT.md` `focus.md` shape.

Storage code lives behind a trait (`FocusRepository` or similar) so later slices can swap or extend without touching parsers. Pure parser separated from I/O.

## Completion promise

On `main`, the popover renders Focuses read from `~/.adhd-ranch/focuses/` and reflects any hand-edit to a `focus.md` within 1 second.

## Acceptance criteria

- [ ] Pure parser reads frontmatter + bullets into a typed `Focus` struct; covered by Rust unit tests with table-driven cases.
- [ ] On launch, app reads `~/.adhd-ranch/focuses/*/focus.md`; popover renders real Focuses.
- [ ] File watcher debounces and emits a Tauri event on add/change/remove; React refreshes within 1s of save (PRD NFR).
- [ ] Hand-edit a `focus.md` (add a `- [ ]` bullet, save) → bullet appears in widget without restart (US5).
- [ ] Empty / missing `~/.adhd-ranch/focuses/` handled gracefully — empty list, no crash.
- [ ] Storage layer exposed via a trait; concrete `MarkdownFocusRepository` injected at composition root.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/002-build-pipeline-and-packaging.md`

## User stories addressed

- User story 1
- User story 5
