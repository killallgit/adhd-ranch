# Contract: Current Product Specification

**Path**: `specs/003-ranch-today/spec.md`

**Status field**: `Draft`

This file is the description of what the ranch does today. It uses the Spec Kit specification shape. It is not a phase history and not an implementation plan.

## Required sections

1. Title, feature branch `003-ranch-today`, created date, status `Draft`, and a short input line that says this specification converts the product requirements document into the current description.
2. User scenarios for a person using the ranch today. Each scenario is independently readable. Scenarios describe current behavior, not a delivery sequence.
3. Edge cases that are true of the product today.
4. Functional requirements for current capabilities. Each carried-forward requirement ends with a source citation and a check note.
5. Limits and non-goals, as requirements a reader can find without opening `PRD.md`.
6. A sources appendix. The appendix is where citations live so the narrative stays readable. Every carried-forward statement in the body has a matching appendix entry.

## Citation

A carried-forward statement uses this form in the appendix:

```text
- <requirement id>: source `<path>` — checked against `main` on <YYYY-MM-DD>. <one sentence of what was confirmed>.
```

A statement that is only true as an accepted decision cites the ADR. A statement that is only a constitution rule cites `.specify/memory/constitution.md`. A statement that is no longer true is omitted, not cited as a requirement.

## Vocabulary

Use the settled word from `CONTEXT.md`. In particular, the specification says Animal, Species, Motion, Focus, Task, Timer, Limit, Over Limit, Harness, Agent Session, Session Activity, Hook Firing, Pen, Agents as Animals, DisplaySpace, Gather, and Settings.

Pig is the Species, and the sprite asset stays pig-named. The specification does not call every Animal a pig. Limit is the soft threshold. Gather is the tray action that brings Animals back onto the primary display. Settings is the stored configuration. Preferences appears only as the macOS menu label for the window that edits Settings.

## Boundaries

- No "Implementation phases" section. Finished phases are not requirements.
- No capability the product no longer has, including a localhost HTTP API and a proposal queue, as a requirement.
- No deferred work as a requirement. Multi-monitor default-on and wrangle stay out of this file.
- The render-domain outcome is one short capability: Focus and Agent Session each project Animals, the shared renderer does not import either domain, and Pig is the only Species. The record of that work is named as `specs/001-render-domain-seam/spec.md`. That spec's requirements are not copied.
- The glossary is not copied. Accepted ADR text is not copied. The specification may cite both.
- No task list, file-edit plan, or Rust or TypeScript design.

## Reader test

A person who opens only this file can list what the ranch does today, what it will not do, and the limits that hold, without `PRD.md`.
