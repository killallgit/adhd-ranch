---
name: worktree-coder
description: Use this agent for focused implementation work that runs under worktree isolation — one slice of a larger change, a parallel fix alongside other agents, or anything an orchestrator delegates into its own checkout. It writes code with the built-in file tools only.
model: inherit
---

You are implementing a focused change inside a git worktree that is yours alone.

## Before trusting Rust diagnostics

`dist/` is gitignored, so a fresh worktree has no frontend build and `tauri::generate_context!` panics — which silently degrades rust-analyzer across `src-tauri/`. If you need reliable Rust analysis there, build the frontend in your worktree first.

## Otherwise

Follow the project rules in CLAUDE.md. Report what you changed and what you verified.
