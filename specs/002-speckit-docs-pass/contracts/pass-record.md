# Contract: Pass Record

**Path**: `specs/002-speckit-docs-pass/pass-record.md`

The pass record is the inventory, the classification, and the unresolved list. It is not a product specification and not an ADR.

## Required sections

1. **How to read this.** Four short rules: `PRD.md` was removed after this pass; historical sources that remain are not requirements; the current description is `specs/003-ranch-today/spec.md` and it is Draft; accepted decisions remain in `docs/adr/` and names remain in `CONTEXT.md`.
2. **Inventory.** One row per source required by FR-001.
3. **Issue classification.** One row per issue that was active at the start of the pass, plus one row per icebox issue, plus one row per closed-without-building note.
4. **Unresolved.** Every disagreement that was not corrected. The section exists even when it says there are none.

## Inventory row

| Column | Content |
|---|---|
| Source | Path |
| Class | Accepted decision, verified observation, present direction, unresolved assumption, or historical context. A mixed document lists each class it contains |
| Checked | What was compared on `main`, or "not a behavior claim" for a pure historical note |
| Disposition | Kept as authority, carried into `specs/003-ranch-today/spec.md`, marked historical, deleted after carry-forward, left deferred, or listed as unresolved |

Minimum sources:

- `PRD.md`
- `issues/README.md` and every issue file under `issues/`, `issues/done/`, and `issues/icebox/`
- `CONTEXT.md`
- `docs/architecture.md`
- `docs/adr/0001-two-apps-over-one-renderer.md`
- `docs/adr/0002-agent-hook-transport.md`
- `docs/adr/0003-rust-owned-render-scene.md`
- `docs/research/serena-multi-agent.md`
- `docs/research/manual-github-release-retention.md`
- `docs/research/claude-codex-session-terminology.md`
- `specs/001-render-domain-seam/spec.md`

Finished issue files may share one inventory row for the archive, with the count of files, plus individual rows for any file whose classification changed during the pass.

`README.md`, `CHANGELOG.md`, and `CLAUDE.md` are not part of the FR-001 inventory. A contradicting sentence found in them is an unresolved entry or a correction noted under Unresolved as "corrected", with the path.

## Issue classification row

| Column | Content |
|---|---|
| Issue | Id and path |
| Class | still to do, already satisfied, superseded, or deferred |
| Evidence | The promise, and whether it is true on `main` |
| Owning specification | One path, or "none — not active" |

## Unresolved entry

```text
### U-<n> <short title>

- Higher: `<path>` — <statement>
- Lower: `<path>` — <statement>
- Check: <what was looked at on main, and why it did not decide>
```

A corrected disagreement is not an unresolved entry. Record corrections in a short **Corrected** list: path, one sentence, higher authority cited.

## Empty unresolved list

When every disagreement was corrected, the section is:

```text
## Unresolved

None.
```
