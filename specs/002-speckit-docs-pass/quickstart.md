# Quickstart: Spec Kit Docs Pass

Validation is a read of the documents. No app launch is required. Run the checks from the repository root after implementation, on a tree synced with current `main`.

## Prerequisites

- `specs/003-ranch-today/spec.md` exists, written from `PRD.md` before that file was deleted.
- `specs/002-speckit-docs-pass/pass-record.md` exists.
- `PRD.md` does not exist.
- `CONTEXT.md`, `docs/architecture.md`, `docs/adr/`, `docs/research/`, `issues/`, and `specs/001-render-domain-seam/` are the sources still under test.

## 1. Cold read of the current specification

Open only `specs/003-ranch-today/spec.md`.

Expected:

- The status is Draft.
- You can name current capabilities, limits, and non-goals from this file alone. `PRD.md` is not in the working tree.
- You do not find a finished implementation-phase list, a localhost HTTP API requirement, or a proposal-queue requirement.
- Deferred multi-monitor default-on and wrangle are absent from the requirements.
- Animals, Limits, Gather, and Settings are named with the glossary's words. Pig is the Species, not the word for every Animal.
- The render-domain outcome points at `specs/001-render-domain-seam/spec.md` and does not copy that spec.
- Every carried-forward requirement has a source citation and a note that it was checked against `main`.

## 2. Active queue

Compare `issues/README.md` with the issue files and with `specs/002-speckit-docs-pass/pass-record.md`.

Expected:

- Every issue that was active at the start (`031`, `051`, `056`, `059`, `060`, `061`, `062`) has one classification row.
- Every file still in `issues/` outside `done/` and `icebox/` is classified still to do.
- Each of those files has one `## Completion promise` and one `## Owning specification`, and no `## Parent PRD`.
- The owning path is `specs/003-ranch-today/spec.md` or one follow-on `specs/<number>-<slug>/spec.md`.
- A follow-on directory contains `spec.md` only.
- `issues/icebox/021-all-monitors-default.md` and `issues/icebox/022-wrangle.md` are still the icebox, and they are not requirements in the product spec.
- The closed-without-building notes still include the issues that were closed that way, unless a correction in the pass record shows the outcome shipped.
- An issue moved to `issues/done/` during the pass starts with **Shipped** or **Closed without building**.

## 3. History and disagreements

Expected:

- `PRD.md` does not exist. `README.md` does not send a reader to it.
- Each file in `docs/research/` has the historical banner.
- `issues/done/README.md` marks the archive historical.
- `CONTEXT.md` still defines the terms. The review banner is gone, or unsettled entries are marked and the governing banner is present.
- `docs/adr/0001-two-apps-over-one-renderer.md`, `docs/adr/0002-agent-hook-transport.md`, and `docs/adr/0003-rust-owned-render-scene.md` are still Accepted and were not rewritten into the product spec.
- Spec 001 has only its lifecycle-status correction from the saved foundation; requirements and validation records are unchanged.
- The pass record has an Unresolved section. Each entry shows both statements and both paths. A disagreement you can still find between the product spec, the glossary, an accepted ADR, and `docs/architecture.md` appears there.

## 4. Behavior did not change

```sh
git diff --stat -- src src-tauri crates package.json package-lock.json Cargo.toml Cargo.lock Taskfile.yaml src/types/generated
```

Expected for the original documentation pass: empty. In the consolidated cleanup, only the five
HTML entrypoint moves are allowed under `src/`; application TypeScript, Rust, generated types,
and dependency manifests remain unchanged. Vite and the unused-source scanner are updated to
follow the new source root.

```sh
task check
```

Expected: the gate passes.

`.specify/feature.json` still names `specs/002-speckit-docs-pass`.
