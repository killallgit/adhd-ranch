# 020 — Drag and toss pigs with physics

## Parent PRD

PRD.md §FR1 (Pig interaction) + Phase 3 polish

## Problem

Pigs wander autonomously but the user has no physical connection to them. Being able to grab, drag, and fling a pig would make the ranch feel alive and tactile.

## What to build

### Drag

- **Mouse down on pig**: freeze pig (already done for click), enter drag mode. Track \`pointermove\` events and update pig position to follow cursor.
- **During drag**: send wide hit-rect (full viewport) so the overlay stays interactive. Show a subtle grab cursor.
- **Mouse up without move** (pure click): behave as today — open PigDetail.
- **Mouse up after drag**: enter toss mode (see below).

Distinguish click vs drag by threshold: if the pointer moved < 4 px from mousedown point, treat as click; otherwise treat as drag+release.

### Toss / light physics

On pointer release after drag, compute velocity from the last few pointer positions (e.g. average delta over last 80 ms). Hand that velocity to \`tickPig\` as the pig's new \`vx/vy\`, scaled to feel natural. The pig coasts, decelerates via a friction constant (e.g. 0.97 multiplier per frame), and bounces off edges as normal.

- **\`src/hooks/usePigMovement.ts\`**: add drag state; expose \`startDrag(pigId)\`, \`moveDrag(x, y)\`, \`endDrag()\` actions. \`endDrag\` computes velocity and injects it into the pig's state. Add friction multiplier applied each tick.
- **\`src/components/PigSprite.tsx\`**: wire \`onPointerDown\`, \`onPointerMove\`, \`onPointerUp\`. Capture pointer on mousedown so events continue during drag.
- Keep \`PigDetail\` opening on clean click (no drag detected).

## Acceptance criteria

- [ ] Clicking (no drag) still opens PigDetail as before.
- [ ] Click-and-drag moves the pig under the cursor in real time.
- [ ] Releasing after a fast drag sends the pig flying in that direction.
- [ ] Pig decelerates naturally (friction) and bounces off screen edges.
- [ ] Pig stays within bounds after toss.
- [ ] \`task check\` green.

## Notes

- Friction coefficient: tune to feel. Start around 0.96–0.98 per rAF frame.
- Velocity cap: clamp toss velocity to \`PIG_SPEED * 6\` so pigs don't teleport.
- This is a delight feature — keep the movement code simple; no full physics engine.

## Blocked by

None — can start after 016 merges and after 018 (hitbox) for consistent interaction area.
