# 012 — Pig overlay (transparent fullscreen canvas + placeholder pigs)

## Parent PRD

`PRD.md` §FR2 (Transparent overlay window), §FR3 (Pig UI)

## What was built

Replaced the floating 400×600 widget with a transparent fullscreen overlay. Pixel pig sprites (placeholder 🐷 emoji) wander the screen — one per Focus. Clicking a pig shows its task list in a detail card. Click-through works for all non-pig screen areas via a Rust polling thread.

## Completion promise

On `main`, the app presents a fullscreen transparent overlay; one pig spawns per Focus and drifts slowly around the screen; clicking a pig shows a dark card with the Focus title and its tasks (each clearable with ✗); clicks on non-pig areas pass through to whatever app is underneath.

## Acceptance criteria

- [x] App window covers the primary monitor (sized to `current_monitor().size()` at startup).
- [x] Window background is fully transparent — underlying apps are visible.
- [x] Clicks on empty screen areas pass through to other apps (verified via `set_ignore_cursor_events`).
- [x] Clicks on pig sprites are captured by the webview.
- [x] Rust polling thread reads `AppHandle::cursor_position()` at ~60fps and toggles click-through.
- [x] Frontend calls `update_pig_rects` Tauri command (every 4 rAF frames) to keep Rust hit-test boxes current.
- [x] One pig per Focus; pig count updates within 1s of focus file change.
- [x] Pigs drift slowly (~35 px/s) with random direction changes every 3–8s; boundary steering keeps them on-screen.
- [x] Animation: 4 frames at ~150ms each; pig bobs slightly.
- [x] Pig label shows Focus title (mirrored correctly when pig faces left).
- [x] Clicking a pig opens `PigDetail` card near the pig (edge-clamped).
- [x] `PigDetail` shows Focus title + task list with ✗ to clear each task.
- [x] Clicking backdrop or pressing Escape closes `PigDetail`.
- [x] `PigSprite.tsx` has a documented swap comment showing exactly where to drop the real sprite sheet.
- [x] `task check` green.

## Blocked by

- `issues/done/011-app-menu-titlebar-always-on-top.md`

## User stories addressed

- US1, US2
