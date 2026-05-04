# Operational info — adhd-ranch

## Build and verify

```sh
task check          # lint + typecheck + tests — must be green before any PR
task test:rust      # Rust tests only (faster)
task test:web       # frontend tests only
task lint           # lint only
task typecheck      # TS typecheck only
```

## Crate testing (isolated)

```sh
cargo test -p adhd-ranch-domain
cargo test -p adhd-ranch-storage
cargo test -p adhd-ranch-commands
cargo test -p adhd-ranch-http-api
```

## Git

```sh
git fetch origin
git checkout main && git pull --ff-only
git checkout -b <branch>
git rebase origin/main          # always rebase, never merge
```

## PR

```sh
gh pr create --title "[NNN-slug]: Title" --body "..."
gh pr merge <number> --squash --delete-branch
```

## Working tree

- Rust crates: `crates/domain/`, `crates/storage/`, `crates/commands/`, `crates/http-api/`
- Tauri host: `src-tauri/src/` (ui_bridge/, app/, display/)
- Frontend: `src/` (components/, hooks/, api/, types/)
- Issues: `issues/NNN-slug.md`
- Ralph plan: `.ralph/PLAN.md`
