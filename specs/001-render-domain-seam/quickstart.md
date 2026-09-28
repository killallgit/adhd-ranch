# Quickstart Validation: Render Domain Seam

Use this guide after implementing the tasks derived from this plan. It proves behavior and the
module boundary; it does not prescribe implementation code.

## Prerequisites

- Node.js 24 and npm
- Stable Rust with `rustfmt` and `clippy` (workspace MSRV is 1.77)
- Task 3.x
- `cargo-shear` (`cargo install --locked cargo-shear` if missing)
- Dependencies installed with:

  ```bash
  task install
  ```

## 1. Verify the Rust Scene, Projections, Regions, and Species Policy

Run the Rust application-layer tests:

```bash
cargo test -p adhd-ranch-commands
```

Expected outcomes:

- Focus projection returns only generated renderer values, emits Pig, and accepts no Agent input.
- Agent projection accepts only harness-neutral Agent Sessions and no Focus, Timer, Claude payload,
  native event, or transport-frame input.
- Equal raw Focus and Session ids produce distinct render ids.
- Focus Timers become fixed or clamped linear absolute-pixel size curves; boundary and
  zero-duration cases are proven.
- Session Activity and Pen changes affect only Agent Animals and renderer regions.
- Region dedupe, ordering, hue, centering, and maximum-size caps preserve current behavior.
- The Pig physical profile preserves current size, speed, friction, turn, toss, and frame cadence.
- `RenderScene` contains no source-domain records.

## 2. Verify Generated Bindings, Scene IPC, and Focus-Only Selection

Regenerate and check the Rust-owned TypeScript contract:

```bash
task gen-types:check
```

Expected outcome: the generated scene values match Rust with no tracked or untracked drift and no
hand-written duplicate Animal contract.

Run selection and App integration tests:

```bash
npm run test -- src/api/tauriRenderSceneReader.test.ts src/hooks/useFocusSelection.test.tsx src/components/App.test.tsx
```

Expected outcomes:

- The scene reader invokes `get_render_scene` and refreshes after Focus, Agent Session, settings,
  and DisplaySpace invalidation without polling on animation frames.
- App renders generated scene Animals and never projects raw Agent Sessions.
- Clicking a Focus Animal opens the matching Focus details.
- Clicking an Agent Animal opens nothing.
- A missing or removed Focus clears the request, frozen id, detail card, and wide interaction area.
- Recreating a Focus with a previously removed id does not silently reopen it.
- The existing tray open-detail event still selects a current Focus.

## 3. Verify the Thin Browser Frame Runtime

Run size sampling, movement, and component tests:

```bash
npm run test -- src/lib/animalSize.test.ts src/hooks/useAnimalMovement.test.ts src/lib/animalMovement.test.ts src/components/App.test.tsx
```

Expected outcomes:

- `regionId: null` uses the full DisplaySpace.
- A known region id confines spawn, walking, resting, and dragging to the region as before.
- An unresolved region id temporarily falls back to DisplaySpace.
- Fixed and linear size curves resolve deterministically before, during, and after their range.
- Sampled size changes do not respawn Animals or restart the animation loop.
- Hit rectangles and concrete Pig rendering consume sampled absolute pixel size.
- Generic movement obtains all physical values from the Rust-generated Species profile; existing
  Pig speed, turning, direction, and animation behavior remain unchanged.
- Drag/toss, Gather, resting, delayed scene/layout availability, and region confinement remain
  unchanged with no per-frame Tauri traffic.

## 4. Check the Boundary Mechanically

Search shared renderer files for forbidden source-domain dependencies:

```bash
rg -n "Focus|Task|Timer|AgentSession|Agent Session|HookFiring|Hook Firing|\bPen\b|kind:" \
  src/lib/animalSize.ts \
  src/lib/animalMovement.ts \
  src/lib/regions.ts \
  src/hooks/useAnimalMovement.ts
```

Expected outcome: no matches. Test descriptions may use source terms, but production shared modules
must not.

Check that generic modules do not define or import Pig-specific policy:

```bash
rg -n "PIG_|PigSprite|pig\.png|sprite.?sheet|frame.?map" \
  src/lib/animalSize.ts \
  src/lib/animalMovement.ts \
  src/lib/regions.ts \
  src/hooks/useAnimalMovement.ts
```

Expected outcome: no matches. Physical Pig policy comes from the generated Rust profile; visual
assets and row mapping belong to `PigSprite`; exhaustive `AnimalSprite` dispatch may name Pig.

Check that source projection exists only in the matching Rust modules:

```bash
rg -n "AgentSession|SessionActivity|Pen" crates/commands/src/focus_animals.rs
rg -n "Focus|FocusTimer|TimerStatus" crates/commands/src/agent_animals.rs
```

Expected outcome: no matches.

Check that obsolete generic names and the union are gone:

```bash
rg -n "projectAnimals|projectFocusAnimals|projectSessionAnimals|FocusAnimal|SessionAnimal|AnimalBase|useAnimalSelection|usePigMovement|RanchAnimal" src
```

Expected outcome: no matches. Pig-specific assets/components and Pig-named Tauri compatibility
wrappers may remain; those exact names are not part of this obsolete-symbol check.

## 5. Run the Repository Gate

```bash
task check
```

Expected outcome: formatting, Clippy debug/release checks, Biome, unused-code checks, TypeScript,
all Rust and Vitest tests, and generated-type drift all pass with a clean generated-types diff.

Windows CI is deferred while the app remains a prototype. Its packaging workflow remains available
for manual dispatch but is not a required PR signal.

The feature is not complete if focused tests pass but `task check` fails.

## 6. Optional Running-App Smoke Test

Confirm no prior dev server owns the strict Vite port:

```bash
lsof -nP -iTCP:1420 -sTCP:LISTEN
```

If the command prints a process, stop that existing dev process from its original shell. Then run:

```bash
task dev
```

With at least one Focus and one live Agent Session visible:

1. Confirm both Animals render and move.
2. Click the Focus; confirm its detail opens and the Animal freezes.
3. Close details; confirm movement resumes and the overlay returns to narrow hit testing.
4. Click the Agent Animal; confirm no Focus detail opens.
5. Start or expire a Focus Timer; confirm only the Focus size/Motion changes.
6. Change Agent Session Activity; confirm only the Agent Motion changes.
7. Confirm the Agent remains in its labeled region while the Focus can use DisplaySpace.
8. Confirm both source domains render through explicit Pig Species dispatch with no visible change.
9. Resize or change enabled displays; confirm Rust-laid-out Agent regions refresh and remain
   centered/capped.

No storage file, settings shape, plugin state, hook protocol, or existing mutation command should
change during this smoke test. The new read-only scene command may run after source, settings, and
DisplaySpace invalidation, but not continuously during ordinary animation.
