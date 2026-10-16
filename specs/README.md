# Feature specifications

| Feature | State | Next step |
| --- | --- | --- |
| [001 — Render Domain Seam](001-render-domain-seam/spec.md) | Implemented in [PR #88](https://github.com/killallgit/adhd-ranch/pull/88) | Use its [validation record](001-render-domain-seam/tasks.md#phase-7-convergence) for renderer changes. |
| [002 — Spec Kit Docs Pass](002-speckit-docs-pass/spec.md) | Documentation converted; new product and follow-on specs remain Draft | Review [the pass record](002-speckit-docs-pass/pass-record.md). |
| [003 — Ranch Today](003-ranch-today/spec.md) | Draft description of the shipped product, including timer chips from PR #89 | Review with the owner before treating it as governing requirements. |
| [004 — Task Timer Hand Edits](004-task-timer-hand-edits/spec.md) | Draft; not implemented | Revisit issue 056's scope. |
| [005 — Late Hook Frame](005-late-hook-frame/spec.md) | Draft; not implemented | Revisit issue 059's scope. |
| [006 — Unheard Turn Ends](006-unheard-turn-ends/spec.md) | Draft; not implemented | Revisit issue 060's scope. |
| [007 — One Hook Delivery](007-one-hook-delivery/spec.md) | Draft; not implemented | Revisit issue 062's scope. |
| [008 — Claude Session Archive](008-claude-session-storage/spec.md) | Approved for planning; design artifacts retained, implementation not started | Revalidate scope, then generate `tasks.md` and shipping issues. |

## Continue the discussion

Start with Ranch Today and the [active issue queue](../issues/README.md). The
[constitution](../.specify/memory/constitution.md), [accepted ADRs](../docs/adr/), and
[glossary](../CONTEXT.md) retain their authority. Saving or renumbering a draft does not approve it.

The archive design was preserved from `chore/project-foundation` as spec 008 because spec 002 is
now the documentation pass. Its [accepted ADR-0004](../docs/adr/0004-local-claude-session-archive.md)
and [four delivery slices](008-claude-session-storage/plan.md#delivery-slices) remain available for
review. Dependency versions, the Rust baseline, and platform support need revalidation before
implementation. The application still uses `.adhd-ranch` and has no archive database.
