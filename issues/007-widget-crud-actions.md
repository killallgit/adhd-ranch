# 007 — Widget actions: new Focus, clear Task, delete Focus

## Parent PRD

`PRD.md`

## What to build

Direct-manipulation actions in the widget so the user can drive the system without `/checkpoint`. Wires the remaining FR4 endpoints to UI controls.

- `+ New Focus` button → form (title + description) → `POST /focuses` → new Focus dir created.
- Inline `✗` next to each Task → `DELETE /focuses/{id}/tasks/{idx}` → bullet removed (no confirm, US4).
- `…` menu on each Focus → `Delete` with confirm dialog → `DELETE /focuses/{id}` → directory removed.
- `POST /focuses/{id}/tasks` available (used internally by accept-flow already; expose for completeness).

Endpoints live behind the same trait architecture as slice 006 — handlers thin, business logic in domain.

## Acceptance criteria

- [ ] `+ New Focus` creates a Focus from the widget; appears in list within 1s.
- [ ] Inline `✗` on a Task removes the bullet; markdown updated atomically; no confirm prompt (US4).
- [ ] Focus delete confirms before removal; directory removed atomically; widget updates.
- [ ] All four endpoints (`POST /focuses`, `DELETE /focuses/{id}`, `POST /focuses/{id}/tasks`, `DELETE /focuses/{id}/tasks/{idx}`) covered by integration tests against a temp dir.
- [ ] Bullet index is 0-based and matches FR4 contract.
- [ ] React components for these actions live in the view layer; HTTP calls confined to the frontend `api` module.
- [ ] `task check` green.

## Blocked by

- Blocked by `issues/006-accept-reject-proposals.md`

## User stories addressed

- User story 3
- User story 4
- User story 5
