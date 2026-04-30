# 008 — Caps + overload alerts

## Parent PRD

`PRD.md`

## What to build

Implement FR6 (caps) and FR7 (configuration) end-to-end.

- Load `~/.adhd-ranch/settings.yaml` at boot; apply defaults for any missing keys.
- Cap detection in the domain layer: pure functions over the catalog state. No side effects.
- Writes that exceed caps still succeed; flip an over-cap flag.
- Widget shows red badge while over.
- macOS system notification fires once per `under → over` transition (`tauri-plugin-notification`). Goes silent on transition back to under.

Cap detection function injected into the widget projection — no globals.

## Completion promise

On `main`, exceeding the configured Focus or Task cap shows a red badge in the widget and fires a one-shot macOS notification; recovering under the cap clears both, with caps configurable via `~/.adhd-ranch/settings.yaml`.

## Acceptance criteria

- [ ] `settings.yaml` parsed; missing keys fall back to PRD defaults (5 / 7).
- [ ] Cap detection unit tests cover under, exactly-at, over, and recovery transitions.
- [ ] With 6 Focuses present, widget shows red badge; reducing to 5 clears it (US6).
- [ ] macOS notification fires once on `under → over`; not re-fired while still over.
- [ ] Notification can be disabled via `alerts.system_notifications: false` and the badge still works.
- [ ] No global state holding over-cap flag — owned by an injected service.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/007-widget-crud-actions.md`

## User stories addressed

- User story 6
