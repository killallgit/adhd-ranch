# 036 — Generate TypeScript types from Rust via ts-rs

## What to build

TypeScript types in `src/types/` are hand-copied from Rust domain structs. If a Rust field changes, TS drifts silently. Replace the manual copies with generated types using the `ts-rs` crate.

### Rust side

Add `ts-rs` to `crates/domain/Cargo.toml`:
```toml
[dependencies]
ts-rs = { version = "10", optional = true }

[features]
export-ts = ["ts-rs"]
```

Annotate each exported struct/enum:
```rust
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export, export_to = "../../src/types/generated/"))]
pub struct Focus { ... }
// Same for: Task, FocusTimer, TimerPreset, TimerStatus, Proposal, ProposalKind,
//           Settings, Caps, Widget, Alerts, MonitorInfo
```

Add a `#[test]` gated on the feature that runs `export_all_to`:
```rust
#[cfg(all(test, feature = "export-ts"))]
#[test]
fn export_ts_bindings() {
    Focus::export_all_to("../../src/types/generated/").unwrap();
    // etc.
}
```

### Frontend side

- Create `src/types/generated/` (gitignored raw output)
- Update `src/types/` barrel imports to re-export from generated
- Remove hand-written type definitions that are now generated

### CI

Add a step to `task check` (or a separate `task gen-types`):
```sh
cargo test -p adhd-ranch-domain --features export-ts
```
CI fails if generated output differs from committed files (diff check).

## Completion promise

TypeScript types for all domain structs are generated from Rust; a Rust type change causes a CI failure rather than a silent runtime drift.

## Acceptance criteria

- [ ] `ts-rs` added as optional dep to `crates/domain`
- [ ] All domain structs/enums annotated with `#[ts]` behind `export-ts` feature
- [ ] `cargo test -p adhd-ranch-domain --features export-ts` generates correct `.ts` files
- [ ] Hand-written `src/types/focus.ts`, `proposal.ts`, `timer.ts` replaced by generated equivalents
- [ ] `src/types/settings.ts` and `src/types/monitor.ts` (added in 032) also generated
- [ ] CI diff-check fails when Rust and TS are out of sync
- [ ] `task check` green

## Blocked by

None — can start immediately.
