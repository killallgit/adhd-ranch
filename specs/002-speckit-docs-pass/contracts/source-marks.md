# Contract: Source Marks

Banners are inserted at the top of the file, under the title when the file has one. Wording is fixed so every marked source reads the same way.

## Product requirements document

Delete `PRD.md` after `specs/003-ranch-today/spec.md` and `specs/002-speckit-docs-pass/pass-record.md` both exist. Do not leave a bannered copy. Do not replace it with a stub.

`README.md` must not tell a reader to open `PRD.md`. Point the design sentence at `specs/003-ranch-today/spec.md`.

Do not edit finished issues only to remove the words `PRD.md`. Replace `## Parent PRD` on active and deferred issues using the owning-spec section below, or, for a deferred issue, with:

```markdown
## Owning specification

None. Deferred. Not a requirement of `specs/003-ranch-today/spec.md`.
```

## Research notes

Insert immediately under the title of each file in `docs/research/`:

```markdown
> **Historical.** This note is evidence. It is not a current requirement and not an accepted decision. The constitution, accepted ADRs, and the current product specification (after owner approval) govern where they disagree with it. Draft specifications are context only. See [specs/002-speckit-docs-pass/pass-record.md](../../specs/002-speckit-docs-pass/pass-record.md).
```

## Finished issue archive

Create `issues/done/README.md`:

```markdown
# Finished issues

> **Historical.** These write-ups record what was asked at the time. They are not current requirements. A file marked **Shipped** had its outcome become true, including when the landed approach differs from the write-up. A file marked **Closed without building** was not built. The active queue is [../README.md](../README.md).
```

Do not add this banner to each existing finished file.

## Shipped note

Add at the top of an issue file moved to `issues/done/` because the promise is already true:

```markdown
> **Shipped.** The completion promise is true on `main`. <One sentence naming the landed approach when it differs from this write-up; otherwise "The write-up matches what landed.">
```

## Closed-without-building note

Add at the top of an issue file moved to `issues/done/` because it was superseded and the outcome was not shipped as that issue's promise:

```markdown
> **Closed without building.** <One sentence naming the replacement or the wrong premise.> This file is not a current requirement.
```

## Glossary banner

`CONTEXT.md` today says the terms are under review. After each entry is read:

- If every entry is settled, replace that blockquote with:

```markdown
> Settled names for the ranch. Specifications, issues, and UI copy use these words. The current product specification does not redefine them.
```

- If any entry is not settled, mark that entry with `**Unsettled**` on its own line under the term, leave its definition as written, and replace the banner with:

```markdown
> Settled entries govern. Entries marked **Unsettled** do not. The current product specification uses only settled names.
```

Do not rewrite settled definitions.

## Owning specification

On each issue that remains to do, replace the whole `## Parent PRD` section with:

```markdown
## Owning specification

`<spec-path>` — <requirement id or section name>
```

`<spec-path>` is `specs/003-ranch-today/spec.md` or the single follow-on spec created for that promise. The completion promise stays one sentence. Do not leave `## Parent PRD` in place beside the new section.

## Render-domain record

Do not add a banner to `specs/001-render-domain-seam/**`. Do not edit those files.
