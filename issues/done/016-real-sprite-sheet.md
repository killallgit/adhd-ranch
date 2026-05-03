# 016 — Real pig sprite sheet

## Parent PRD

`PRD.md` §FR3 (Pig UI — sprite animation)

## What to build

Swap the placeholder 🐷 emoji in `PigSprite.tsx` for the real 4-direction × 4-frame pixel-art sprite sheet PNG. Wire CSS `background-position` to the current `direction` and `frame` props so pigs animate correctly as they walk.

**HITL gate**: the user must supply the sprite sheet PNG and confirm the frame layout (dimensions per frame, row order for directions) before code changes begin.

- **Asset placement**: drop the sprite sheet at `src/assets/pig-spritesheet.png`. Vite will bundle it and expose it as a hashed URL via `import pigSheet from '../assets/pig-spritesheet.png'`.
- **Frame layout** (confirm with user before coding):
  - Default: 4 rows × 4 columns. Row order: `front=0, right=1, back=2, left=3` (matches `PigSprite.tsx` `DIRECTION_ROW` mapping).
  - Frame width = `sheet_width / 4`, frame height = `sheet_height / 4`.
- **`PigSprite.tsx` change** (the swap comment at lines 16–26 marks the exact location):
  - Remove the `<span className="pig-emoji">` element.
  - Replace with:

    ```tsx
    <div
      className="pig-sprite-frame"
      style={{
        backgroundImage: `url(${pigSheet})`,
        backgroundPosition: `-${frame * FRAME_W}px -${row * FRAME_H}px`,
        width: FRAME_W,
        height: FRAME_H,
        imageRendering: "pixelated",
      }}
    />
    ```

  - `row` derived from `direction`: `left=1, right=2` (up/down unused in v1.2 — pigs only move in 2D horizontal plane for now).
  - Remove `scaleX(-1)` transform from the button (direction handled by row selection instead).
- **CSS**: add `.pig-sprite-frame { display: block; }`. Remove `.pig-emoji` rule.
- **`PIG_SIZE`** in `usePigMovement.ts`: update to match `FRAME_H` (the sprite's actual pixel height × scale factor if rendering at 2×).

## Completion promise

On `main`, pigs render as pixel-art sprites from the real sprite sheet, walking left or right with correct frame animation, at the same hitbox size as the placeholder.

## Acceptance criteria

- [ ] User has provided the sprite sheet PNG and confirmed frame dimensions + row order. (HITL gate — must happen before code starts.)
- [ ] Pigs display the sprite sheet frames instead of the emoji.
- [ ] Walking right uses the correct sprite row; walking left uses a different row (not a CSS flip of right).
- [ ] Animation cycles through 4 frames at ~150ms per frame.
- [ ] `PIG_SIZE` in `usePigMovement.ts` matches the rendered sprite height so the Rust hit-test box aligns with the visible pig.
- [ ] `imageRendering: pixelated` applied so sprites don't blur at display scale.
- [ ] `task check` green.

## Blocked by

None (code-side). Requires user to supply sprite sheet asset before implementation begins.

## User stories addressed

- US1 (pigs are visually recognisable pixel-art animals, not emoji)
