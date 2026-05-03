# 019 — PigDetail card redesign: editable, opaque, larger

## Parent PRD

PRD.md §FR1 (focus detail card) + Phase 3 polish

## Problem

The current PigDetail card has three pain points:
1. No way to add tasks inline — user must use the HTTP API or hand-edit markdown.
2. Background is translucent/transparent, making text hard to read over busy desktop backgrounds.
3. Card is slightly too compact — padding feels tight and the card needs to grow with longer task lists.

## What to build

### Always-editable task input

Add a text input at the bottom of the card. Pressing Enter (or a "+" button) appends a new task via the existing \`append_task\` Tauri command. The field clears after submit. This makes the card self-contained for adding tasks without leaving the overlay.

- **\`src/components/PigDetail.tsx\`**: add a controlled \`<input>\` with placeholder "Add task…". On Enter/submit: call \`onAddTask(text)\`, clear field.
- **\`src/components/App.tsx\`**: wire \`handleAddTask\` → \`focusWriter.appendTask(selectedFocus.id, text)\`.

### Opaque background

Replace the current semi-transparent backdrop style with a solid opaque card background (dark, e.g. \`rgba(20, 20, 20, 1)\` or a design-system token). The backdrop overlay behind the card (click-catcher) can remain low-opacity.

### Size + padding

- Increase card min-width from current value to ~340 px.
- Increase internal padding from current to 16 px (was 12 px or less).
- Task list max-height with overflow-y scroll so long task lists don't overflow the card.

## Acceptance criteria

- [ ] Card has an "Add task…" input at bottom; Enter appends task and clears field.
- [ ] Card background is fully opaque.
- [ ] Card min-width ≥ 340 px, internal padding ≥ 16 px.
- [ ] Long task lists scroll within the card rather than overflowing.
- [ ] Click-outside and Escape still close the card.
- [ ] \`task check\` green.

## Blocked by

None — can start after 016 merges.
