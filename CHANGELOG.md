# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Tauri v2 + React scaffolding with borderless always-on-top tray popover.
- Build pipeline: `.app` and `.dmg` produced from tag-driven release workflow.
- Markdown-backed focuses on disk, watched for live UI refresh.
- Localhost HTTP API exposing `/health` and `/focuses`.
- `/checkpoint` slash command and a proposal queue for accept/reject review.
- Atomic write of accepted proposals with a decision audit log.
- Widget CRUD: new focus, delete focus, add and remove tasks.
- `settings.yaml` with focus and task caps; cap badge and macOS overload notifications.
- Edit-proposal modal, empty-state hero, and v1 README.

### Changed

- Cap notifications now flow through a `CapNotifier` trait owned by the commands crate; the Tauri shell provides a single adapter, removing the inline cap-evaluation/notification logic from the app composition root.
- Proposal accept/reject now lives in a dedicated `ProposalLifecycle` module that owns proposal load → edit → validate → mutate → record-decision → clear-queue. The `ProposalDispatcher`/`*Applier` strategy traits are removed; the inline `match` over `ProposalKind` replaces the per-kind adapters. Focus creation in the direct path and the proposal path now share a single helper.
- Tauri filesystem watchers now share a single `install_change_handlers` helper that fans a debounced change event out to a list of handlers, replacing the asymmetric mix of inline closure and per-watcher helper.
- React widget composes its three async data sources through a single `useAppState` hook with one readiness contract; proposal-reader errors now surface in the UI instead of being silently swallowed.
