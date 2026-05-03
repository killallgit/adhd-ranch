# 018 — Larger pig hitbox

## Parent PRD

PRD.md §FR2 + Phase 3 polish

## Problem

The pig sprite is 48×48 CSS px. Clicking it precisely is fiddly — especially on pixel-art tails, ears, and corners where the sprite is mostly transparent. Users miss the click and interact with the desktop instead.

## What to build

Add a separate `HITBOX_PADDING` constant in \`usePigMovement.ts\` (e.g. 16 px) and use \`PIG_SIZE + HITBOX_PADDING\` when computing the rects sent to Rust via \`update_pig_rects\`. The visual sprite size stays unchanged; only the hit-detection footprint grows.

- `src/hooks/usePigMovement.ts`: add `export const HITBOX_PADDING = 16`; in `syncRects` use `size: (PIG_SIZE + HITBOX_PADDING) * dpr` and offset `x`/`y` by `-(HITBOX_PADDING / 2) * dpr` so the hitbox is centred on the sprite.
- No Rust changes needed — \`PigHitTester::is_hit\` already works on whatever rects it receives.
- \`PigSprite\` click handler already fires on the \`<button>\` element; the button's visible area stays 48 px. The extra hit area is only for the Rust polling thread (click-through toggle), not for the React click event.

## Acceptance criteria

- [ ] Clicking anywhere within ~8 px outside the pig sprite still registers as a pig click.
- [ ] Visual sprite size unchanged.
- [ ] \`task check\` green.

## Blocked by

None — can start after 016 merges.
