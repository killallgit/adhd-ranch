# 034 — Move Focus/Task invariants into domain

## What to build

Two domain rules currently live in `crates/commands/src/focus.rs`:
- `title.trim().is_empty()` check in `Commands::create_focus`
- `text.trim().is_empty()` check in `Commands::append_task`

These are domain invariants that belong in `crates/domain/`. Move them there as typed constructors. Commands drops its own validation guards.

### Domain changes (`crates/domain/`)

Add validated constructors:

```rust
impl NewFocus {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Result<Self, DomainError>
    // Err if title.trim().is_empty()
}

pub struct TaskText(String);
impl TaskText {
    pub fn new(text: impl Into<String>) -> Result<Self, DomainError>
    // Err if text.trim().is_empty()
    pub fn as_str(&self) -> &str
}
```

Add `DomainError` if not already present:
```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("task text must not be empty")]
    EmptyTaskText,
}
```

### Commands changes

Remove the `trim().is_empty()` guards in `Commands::create_focus` and `Commands::append_task`. Use `NewFocus::new(...)` and `TaskText::new(...)`. Map `DomainError` to `CommandError::BadRequest`.

### Tests

Domain tests own the invariant assertions. Commands tests drop the validation assertions.

## Completion promise

Focus title and task text invariants are enforced once in `crates/domain/` via typed constructors; no validation logic remains in the commands layer.

## Acceptance criteria

- [ ] `NewFocus::new` returns `Err(DomainError::EmptyTitle)` for blank title
- [ ] `TaskText::new` returns `Err(DomainError::EmptyTaskText)` for blank text
- [ ] Domain has tests for both error cases
- [ ] `Commands::create_focus` and `Commands::append_task` contain no `trim().is_empty()` guards
- [ ] Existing acceptance criteria for create_focus and append_task commands still pass end-to-end
- [ ] `task check` green

## Blocked by

None — can start immediately.
