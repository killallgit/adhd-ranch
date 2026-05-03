# 031 — Notification settings in tray

## Parent PRD

PRD.md §FR7 (configuration)

## What to build

Expose per-source notification toggles in the tray menu, using the `NotificationSource` interface from 029. Designed to slot into the future settings window (026) without rework.

### Tray structure

```
Settings
  ├── Displays          (existing)
  ├── Window
  │     └── Always on Top  (existing)
  └── Notifications
        └── Timer expired  ✓/✗  (toggleable, sourced from TimerExpiredSource::label())
```

- Toggle reads/writes `NotificationSettings::sources["timer_expired"]`
- Persists immediately to `settings.yaml`
- Adding a new `NotificationSource` implementation automatically appears here — no hardcoded list

### Implementation note

The tray submenu builder should iterate registered `NotificationSource` implementations (or a static list for now) rather than hardcoding menu items. This makes the submenu extensible when new sources are added.

## Completion promise

User can enable/disable timer expiry notifications from the tray without editing `settings.yaml`. The mechanism is generic enough that new notification sources appear automatically.

## Acceptance criteria

- [ ] Tray → Settings → Notifications submenu exists
- [ ] "Timer expired" toggle reflects current `notifications.sources["timer_expired"]` value
- [ ] Toggling updates `settings.yaml` and takes effect immediately (no restart needed)
- [ ] Existing Settings submenus (Displays, Window) unaffected
- [ ] Adding a second `NotificationSource` requires no changes to tray menu builder
- [ ] `task check` green

## Blocked by

029 (NotificationSource trait + NotificationSettings in domain)
