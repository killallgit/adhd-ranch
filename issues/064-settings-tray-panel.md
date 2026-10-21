# Settings tray panel

## Owning specification

Small bug fix authorized by the current user request: Settings is too small, its controls overflow, and it must stay above other windows attached to the Ranch menu-bar icon.

## What to build

Verified current observations: the host creates a fixed 380×300 window; the frontend tries to resize it without a window resize capability; the global viewport clips content. Use a screen-bounded native Settings window anchored beneath the tray icon on macOS, with always-on-top independent of the Widget preference. Reopening must restore the anchor. Keep existing preferences and give the content a vertical scroll area that supports long display names and many displays. Other platforms use their tray geometry when available and a work-area fallback otherwise.

Constitution compliance: retain local state, existing settings contracts, domain separation, and current dependencies; put native window effects in the host and keep the component view-only. This issue adds no IPC capabilities or settings writes.

## Completion promise

Opening Settings shows every control inside a usable, vertically scrollable panel that stays above other windows and reopens beneath the Ranch menu-bar icon on macOS, with screen-bounded placement on other platforms.

## Acceptance criteria

- [ ] A native macOS check observes Settings beneath the Ranch icon, above other windows, and reanchored on repeated opens, independently of Widget Always on Top.
- [ ] A real rendered-window check observes no horizontal clipping and scroll/keyboard access to every control with many long display labels at 380×300 and the default panel size.
- [x] Geometry regressions cover negative monitor origins, mixed DPI, right-edge icons, short work areas, and top/bottom tray placement and fallback.
- [x] Existing setting updates and keyboard order pass component tests.
- [x] `task check` and `task check:windows` pass after synchronization with current main.

Native app launch and personal settings changes are outside this session's verification scope. Browser automation is unavailable in this session; native stacking, tray placement, and rendered scrolling remain unverified until an authorized GUI check. Do not mark this issue done before those checks and the normal merge gate.

## Verification

- `task check`: exit 0 after rebasing onto main `67140af`; 342 Rust tests (60 host tests) and 179 frontend tests passed, including six geometry checks and keyboard access through every dynamic notification toggle and many displays.
- `task check:windows`: exit 0. Existing unrelated warning: unused `script` in `src-tauri/src/app/claude_plugin/contract.rs:94`; the cross-check uses the expected placeholder sidecar.
- `git diff --check`: exit 0.

## Blocked by

None for implementation. Native and rendered-window acceptance checks remain pending.

## User stories addressed

Reach and edit all preferences, and find Settings beside the Ranch menu-bar icon whenever it opens.
