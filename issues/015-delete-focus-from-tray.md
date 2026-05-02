# 015 — Delete Focus from tray

## Parent PRD

`PRD.md` §FR4 (Menu bar item — focus management)

## What to build

Each Focus item in the tray NSMenu gains a submenu. Hovering the Focus title reveals an arrow; the submenu contains a single "Delete…" item. Clicking "Delete…" shows a macOS native confirmation dialog; confirming calls `delete_focus` and the pig disappears from the overlay.

- **Tray menu change** (`src-tauri/src/app/tray.rs`):
  - Replace each plain `MenuItem` for a focus with a `Submenu` whose title is the focus title.
  - Submenu contents:
    ```
    Delete "Customer X bug"…
    ```
  - Menu item id carries the focus id so the event handler knows which focus to delete.
- **Confirmation dialog**:
  - Use `tauri_plugin_dialog::blocking::confirm` (or the async variant) to show a native macOS sheet:
    - Title: `Delete Focus?`
    - Message: `"Customer X bug" will be permanently removed.`
    - Buttons: Delete (destructive), Cancel.
  - Only call `delete_focus` if the user confirms.
- **After delete**:
  - `delete_focus` command removes the focus directory.
  - File watcher fires `focuses-changed`; tray menu rebuilds (from 012) and pig disappears (existing overlay behaviour).
- **Plugin**: add `tauri-plugin-dialog` to `Cargo.toml` and `tauri.conf.json` capabilities if not already present.

## Completion promise

On `main`, hovering a Focus name in the tray menu reveals a submenu; clicking "Delete…" shows a native confirmation dialog; confirming removes the focus, updates the menu, and removes the pig from the overlay within 1s.

## Acceptance criteria

- [ ] Each Focus item in the tray menu has a submenu arrow.
- [ ] Submenu contains "Delete \"{title}\"…" item.
- [ ] Clicking it shows a native macOS confirmation dialog.
- [ ] Confirming deletes the focus directory and calls the existing `delete_focus` command path.
- [ ] Cancelling does nothing.
- [ ] Tray menu updates within 1s of deletion (pig also disappears from overlay).
- [ ] No confirmation dialog if the user cancels.
- [ ] `task check` green.

## Blocked by

- `issues/013-tray-icon-focus-list.md`

## User stories addressed

- US3 (delete a stale Focus from the menu)
