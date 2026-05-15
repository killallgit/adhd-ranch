# 048 — FocusDocument mutation module

## Parent PRD

CONTEXT.md §Persistence + CLAUDE.md §Functional-first and §Data/view separation

## What to build

Deepen the markdown persistence implementation by extracting pure Focus document mutation out of `MarkdownFocusStore`.

Today `crates/storage/src/focus_store.rs` owns both disk I/O and the rules for editing `focus.md`: replacing the frontmatter title, appending Task bullets, deleting indexed Task bullets, updating Task text, and toggling Task status. Those are document rules, not filesystem rules.

### Target shape

Add a pure module in `crates/storage`, for example `focus_document.rs`:

```rust
pub struct FocusDocument {
    raw: String,
}

impl FocusDocument {
    pub fn parse(raw: String) -> Result<Self, FocusDocumentError>;
    pub fn rename_focus(&self, title: &str) -> Result<String, FocusDocumentError>;
    pub fn append_task(&self, text: &str) -> Result<String, FocusDocumentError>;
    pub fn delete_task(&self, index: usize) -> Result<String, FocusDocumentError>;
    pub fn update_task(&self, index: usize, text: &str) -> Result<String, FocusDocumentError>;
    pub fn toggle_task(&self, index: usize, done: bool) -> Result<String, FocusDocumentError>;
}
```

Exact names may change. The interface should expose document operations and return the next raw document. It should not read or write files.

`MarkdownFocusStore` should reduce to locating `focus.md`, reading raw text, calling `FocusDocument`, and writing the returned raw text atomically.

## Completion promise

All `focus.md` mutation rules live in a pure FocusDocument module; `MarkdownFocusStore` owns file placement and atomic writes, not line-editing logic.

## Acceptance criteria

- [x] A pure FocusDocument module exists under `crates/storage/src/`
- [x] FocusDocument has unit tests for rename, append, delete, update, toggle, missing Task index, checked and unchecked Task preservation, and trailing-newline preservation
- [x] `MarkdownFocusStore::{rename_focus, append_task, delete_task, update_task, toggle_task}` delegate document mutation to FocusDocument
- [x] Existing `MarkdownFocusStore` tests still cover disk placement, missing Focus behavior, and atomic write integration
- [x] No parsing or mutation logic is moved into `crates/domain`; markdown shape stays a storage concern
- [x] `task check` green

## Blocked by

None

## User stories addressed

- "When markdown editing rules change, I can test them without filesystem setup and without touching every storage method."
