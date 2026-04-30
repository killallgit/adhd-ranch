---
description: Capture what you just did into adhd-ranch. Picks the right Focus or proposes a new one.
allowed-tools: ["Bash"]
---

You are routing the user's current work into their adhd-ranch app.

## Step 1 — locate the running app

```bash
PORT_FILE="$HOME/.adhd-ranch/run/port"
if [ ! -f "$PORT_FILE" ]; then
  echo "adhd-ranch not running, please start the app"
  exit 1
fi
PORT=$(cat "$PORT_FILE")
if ! curl -fs "http://127.0.0.1:$PORT/health" >/dev/null; then
  echo "adhd-ranch not running, please start the app"
  exit 1
fi
echo "PORT=$PORT"
```

If either guard fails, **stop and report the error to the user**. Do not invent a port.

## Step 2 — fetch the catalog

```bash
curl -s "http://127.0.0.1:$PORT/focuses"
```

Returns `[{ id, title, description }, …]`.

## Step 3 — decide one of three kinds

Compose ONE sentence (≤ 12 words) summarizing what we just did in this session.

Pick the kind:

- **`add_task`** — the work clearly belongs under one existing Focus.
  Required fields: `target_focus_id`, `task_text`, `summary`, `reasoning`.
- **`new_focus`** — the work doesn't fit any existing Focus and deserves its own bucket.
  Required fields: `new_focus: { title, description }`, `summary`, `reasoning`.
- **`discard`** — the work isn't worth tracking (dead-end, throwaway, exploration).
  Required fields: `summary`, `reasoning`.

`reasoning` is one sentence: why this Focus, or why nothing fits, or why discard.

Limit yourself to **one** proposal per `/checkpoint`. If multiple buckets are affected, the user will run it again.

## Step 4 — POST the proposal

Build a JSON body matching the kind, then:

```bash
curl -fsS -X POST "http://127.0.0.1:$PORT/proposals" \
  -H 'content-type: application/json' \
  -d "$BODY"
```

A `201 Created` with `{ "id": "…" }` means the proposal is queued. Report the id back to the user, briefly. Do **not** describe what they should do next — they accept or reject in the widget.

If the response is `400`, surface the error message verbatim and stop.

## Boundaries

- Never call any LLM yourself; you are the LLM.
- Never write to `~/.adhd-ranch/` directly. Only the HTTP API.
- Never propose more than one item per invocation.
