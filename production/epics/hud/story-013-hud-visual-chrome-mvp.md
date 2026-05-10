# Story 013: HUD Visual Chrome MVP — Timer + Class Figurines + RESOLUTION Dim

> **Epic**: HUD
> **Story ID**: S10-POLISH-001
> **Status**: Not Started
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 10 active

## Context

This story is the Sprint 10 Must Have visual-chrome MVP for the HUD. PAW-004
(`a7e397a`, merged at `2132129`) wired the HUD's two `HudFigurine` markers
(own + opponent), the `HudTimerBar` marker, and ten objective-dot `ImageNode`
entities to placeholder/path-constant handles inside `client/src/ui/hud/`. The
visible HUD chrome the friend-game route shows today is therefore: figurine
slots present but stuck on `PlaceholderAssets.fallback`, timer bar present but
inert, dim overlay absent. This story consumes the PAW-004-wired entities and
the `client/src/asset_wiring.rs` constants so the friend-game route shows:

1. The phase timer bar with the wired sprite from PAW-004 visible during
   timed phases (no countdown numerals — see HUD-11).
2. Opponent and own class figurines updated from
   `PresentationGameSnapshotMessage` class data via the
   `sync_figurine_image_system` already introduced by PAW-004-d.
3. A RESOLUTION-phase dim/freeze overlay specified by `design/gdd/hud.md`
   that renders only while `Phase::Resolution`, lifts on
   `S2CPhaseChanged(DRAFT_SHOP|DRAFT_AUCTION|GAME_OVER)`, and never adds
   client-side phase authority.

The story does **not** add a countdown numeral (HUD-11 forbids it), does not
add a new C2S message, does not add a second `S2CPhaseChanged` drain, does
not claim Standard-tier accessibility completion (QA-COND-0005 remains
accepted-risk friend-game scope), and does not author new sprite assets
(per Sprint 10 risk row 151: "constrain each to wiring already-approved
sprites from `asset_wiring.rs`; no new asset authoring is in scope").

**Primary sources**:

- `production/sprints/sprint-10.md` (S10-POLISH-001 row, line 95).
- `design/gdd/hud.md` Rule 9 (RESOLUTION persistence), Rule 10 (GAME_OVER
  freeze), HUD-09 / HUD-10 / HUD-11 / HUD-18 / HUD-19 acceptance blocks.
- `production/epics/hud/EPIC.md` lines 46–49 (TR-HUD-006 RESOLUTION
  persistence, TR-HUD-009 FROZEN mode tiebreak).
- `production/epics/presentation-asset-wiring/story-004-hud-figurines-timer-dots.md`
  (PAW-004 — wired the figurine, timer bar, and dot `ImageNode` entities
  this story consumes).
- `client/src/asset_wiring.rs` (`HUD_PHASE_TIMER_BAR_ASSET` and class
  figurine path constants — already on `main` at `a7e397a`).
- `client/src/ui/hud/mod.rs` (`HudFigurine`, `HudTimerBar`,
  `sync_figurine_image_system`, `sync_dot_image_on_objective_destroyed_system`
  — already present at `2132129`).

**GDD, UX, and TR trace**:

- GDD: `design/gdd/hud.md` Rule 9 — RESOLUTION: HUD persists, sister UIs
  vanish; HUD-09 BLOCKING acceptance.
- GDD: `design/gdd/hud.md` Rule 10 — GAME_OVER: HUD freezes, never reveals
  identity retroactively; HUD-10 BLOCKING acceptance.
- GDD: `design/gdd/hud.md` HUD-11 BLOCKING — no countdown numerals (the
  timer bar is a sprite-only chrome readout for this story).
- GDD: `design/gdd/hud.md` HUD-18 / HUD-19 BLOCKING — phase transitions
  into and out of RESOLUTION.
- TR: **TR-HUD-006** — RESOLUTION persistence (HUD root visible while sister
  UIs hide; enforced via `PresentationSet` ordering). Source:
  `production/epics/hud/EPIC.md:46`.
- TR: **TR-HUD-009** — FROZEN mode on GAME_OVER (incremental updates rejected;
  `S2CGameSnapshot` bypasses FROZEN and re-applies). Source:
  `production/epics/hud/EPIC.md:49`. The dim/freeze overlay introduced by
  this story is the visual surface of the FROZEN-mode rule for the
  RESOLUTION sister-UI hiding window.

**ADR Governing Implementation**:

- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
  — PRIMARY. `HudPlugin` 4th in `PresentationPlugin`; `PresentationSet`
  (PhaseTransition → MessageDrain → StateSync → AnimationTick); 18→21
  pre-pooled entities (post-PAW-004); the dim overlay added here MUST be
  pre-pooled at session entry, not spawned per-update.
- [ADR-011: Reconnect + Snapshot](../../../docs/architecture/adr-011-reconnect-and-snapshot.md)
  — `S2CGameSnapshot` triggers full HUD rebuild; FROZEN-mode tiebreak —
  snapshot always wins, FROZEN re-applies. The dim overlay's visibility
  must rebuild from snapshot (overlay visible iff snapshot phase ==
  RESOLUTION at restore moment).
- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
  — HUD is read-only; zero C2S messages; no client-side phase mutation.
  Forbidden: any system in this story writing to `CurrentClientPhase` or
  emitting a synthetic `S2CPhaseChanged`.
- [ADR-001: Objective Identity Unicast](../../../docs/architecture/adr-001-objective-identity-unicast.md)
  — `HudObjectiveUpdate` strips `was_fake` at Board Rendering boundary;
  this story does not touch the dot pipeline but inherits the constraint.
- [ADR-008: Lightyear Channel Config](../../../docs/architecture/adr-008-lightyear-channel-config.md)
  — `S2CPhaseChanged` is on `ReliableChannel`, drained by the existing
  `MessageDrain` set's single `phase_sink_system`; this story does not
  add a second drain.

**Engine**: Bevy 0.18 + Lightyear 0.26 (browser/WASM client target) | **Risk**: MEDIUM

**Engine Notes**:

- Activate `liv-bevy-018` for any `.rs` change. HUD nodes use Bevy 0.18
  Required Components — `Node { ... }` directly; never `NodeBundle`.
- Sprite handles for the timer bar and figurines are `ImageNode { image,
  .. }` (already in place at `a7e397a`). The dim overlay is a screen-space
  `bevy_ui` `Node` with a translucent `BackgroundColor` — not a world-space
  sprite, not a render layer toggle.
- All path constants come from `client/src/asset_wiring.rs`; no inline
  asset path strings are added (control-manifest forbidden rule:
  "Path constants from `asset_wiring.rs`").
- Phase truth is read from `Res<CurrentClientPhase>` populated by the
  existing single-drain `phase_sink_system` (per ADR-021 + ADR-008). The
  overlay's visibility system reads this resource — it never reads
  `MessageReceiver<S2CPhaseChanged>` directly.
- Visibility flips are instantaneous (HUD-12b BLOCKING) — no
  `bevy_tweening` `Animator<T>` is attached to the dim overlay.
- Bevy 0.18: `Visibility::Visible` / `Visibility::Hidden` on the overlay
  root is the correct toggle; do not manipulate `BackgroundColor.alpha` per
  frame, which would push the overlay onto the AnimationTick set unnecessarily.

**Control Manifest Rules (2026-05-05)**:

- Required: HUD chrome uses `ImageNode` for sprite surfaces, not `Sprite`.
- Required: Path constants from `asset_wiring.rs` (figurines, phase timer
  bar, objective dots) — established by PAW-004; this story adds zero new
  inline asset paths.
- Required: Figurine handle synced from authoritative
  `PresentationGameSnapshotMessage` class — established by PAW-004-d; this
  story relies on that sync, does not modify it.
- Required: Presentation steady-state work stays below 1 ms/frame; phase
  boundary spikes stay below 3 ms.
- Required: HUD presentation entities pre-pooled at session entry — the
  RESOLUTION dim overlay root MUST spawn at HUD pool init, not on phase
  entry.
- Required: `S2CPhaseChanged` is drained by the existing single
  `phase_sink_system` populating `Res<CurrentClientPhase>`; new HUD systems
  read that resource only.
- Forbidden: Sprite for HUD surfaces (use `ImageNode`).
- Forbidden: Client presentation must not assert or mutate authoritative
  game state — no system in this story may write to `CurrentClientPhase`,
  emit a synthetic `S2CPhaseChanged`, or add a second
  `MessageReceiver<S2CPhaseChanged>` drain.

---

## Scope

### In Scope

- Wire the existing PAW-004-spawned `HudFigurine` (own + opponent) so the
  visible friend-game route shows the class-specific path on
  `PresentationGameSnapshotMessage` rather than only the placeholder
  fallback. (`sync_figurine_image_system` already exists per PAW-004-d;
  this story closes any gap that prevents the wired path from rendering
  on the live client route — for example, ensuring the system is
  registered in `HudPlugin` and runs in the `PresentationSet::StateSync`
  schedule per ADR-021.)
- Wire the existing PAW-004-spawned `HudTimerBar` `ImageNode` so the
  phase timer bar sprite from `HUD_PHASE_TIMER_BAR_ASSET` is visible
  during timed phases (DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION,
  PLACEMENT) and hidden during RESOLUTION and GAME_OVER, per the GDD's
  "no countdown numerals" rule (HUD-11) and Rule 9's RESOLUTION
  persistence.
- Add a single pre-pooled RESOLUTION dim/freeze overlay root entity
  (`HudDimOverlay` marker) under the HUD root, spawned at HUD session
  entry alongside the existing 21 PAW-004-era entities. The overlay is a
  full-viewport translucent `Node` with `BackgroundColor` set to a dim
  alpha and `Visibility::Hidden` at session entry.
- Add a single visibility-control system
  (`sync_dim_overlay_for_resolution_system`) that reads
  `Res<CurrentClientPhase>`, sets `Visibility::Visible` while
  `Phase::Resolution`, and sets `Visibility::Hidden` for every other
  phase. The system is registered in `HudPlugin` and runs in the
  `PresentationSet::StateSync` schedule per ADR-021.
- Add an integration test
  `tests/integration/hud/hud_resolution_dim_test.rs` that asserts the
  dim overlay's `Visibility` flips correctly across PLACEMENT →
  RESOLUTION → DRAFT_SHOP and PLACEMENT → RESOLUTION → GAME_OVER
  transitions, and that no system mutates `CurrentClientPhase` from the
  client side.
- Update `HUD_ENTITY_COUNT` from 21 → 22 to account for `HudDimOverlay`,
  and update `tests/integration/presentation/asset_wiring_foundation_test.rs`
  to assert the new count (mirrors PAW-004-f's pattern).
- Author the manual evidence document at
  `production/qa/evidence/sprint-10-hud-chrome-evidence.md` recording
  the friend-game-route screenshot capture (timer bar visible during
  timed phases, figurines showing class art, dim overlay visible during
  RESOLUTION, dim overlay hidden during GAME_OVER FROZEN).

### Out of Scope

- No countdown numerals on the timer bar (HUD-11 BLOCKING — this story
  cannot add them).
- No client-side optimistic phase authority. No system added by this
  story may write to `CurrentClientPhase`, emit a synthetic
  `S2CPhaseChanged`, or add a second `MessageReceiver<S2CPhaseChanged>`
  drain. The existing `phase_sink_system` remains the single source of
  phase truth.
- No new sprite assets. `HUD_PHASE_TIMER_BAR_ASSET` and the class
  figurine constants from PAW-004 are the wired surfaces. New asset
  authoring is out of scope per Sprint 10 risk row 151.
- No claim of Standard-tier accessibility completion. QA-COND-0005
  remains accepted-risk friend-game scope. The dim overlay is not a
  contrast/colorblind remediation; it is a phase-state chrome cue
  specified by the GDD.
- No closure of QA-COND-0005, QA-COND-0006, S8-QA-001-W1, or any other
  Sprint 9 carry condition.
- No claim of public-release readiness, full playable-client manual QA,
  full game completion, or final visual polish.
- No tween or animation on the dim overlay (HUD-12b BLOCKING —
  visibility flips are instantaneous; no `Animator<T>` is attached to
  the overlay root).
- No changes to Hand UI, Shop/Auction UI, board content, scoreboard
  dots, gold labels, mana labels, phase label, round counter, or any
  HUD zone outside the timer bar, figurines, and the new dim overlay.
- No changes to network protocol, lightyear channels, or any C2S /
  S2C message type.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-10.md:95` S10-POLISH-001 row.)

- [ ] **Wired phase timer bar visible**: GIVEN a friend-game session in
      a timed phase (DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, or
      PLACEMENT), WHEN the HUD renders, THEN the `HudTimerBar`
      `ImageNode.image` resolves to `HUD_PHASE_TIMER_BAR_ASSET` (not
      `PlaceholderAssets.fallback`) and the entity's parent chain has
      `Visibility::Visible` at the HUD root. *Evidence target*:
      `tests/integration/hud/hud_resolution_dim_test.rs` —
      `test_timer_bar_visible_in_timed_phases` sub-test reads
      `HudTimerBar` query and asserts `ImageNode.image == handle from
      asset_wiring::HUD_PHASE_TIMER_BAR_ASSET`.
- [ ] **Wired class figurines visible**: GIVEN
      `PresentationGameSnapshotMessage` arrives with `own_class=Iop`
      and `opponent_class=Cra`, WHEN HUD state sync completes, THEN the
      own and opponent `HudFigurine` `ImageNode.image` handles each
      resolve to the class-specific asset constant from
      `client/src/asset_wiring.rs` (not `PlaceholderAssets.fallback`).
      *Evidence target*: existing `tests/integration/presentation/hud_asset_wiring_test.rs`
      already covers `sync_figurine_image_system` per PAW-004-d; this AC
      is satisfied by adding a friend-game-route assertion in the new
      `hud_resolution_dim_test.rs` (or by extending the PAW-004 test
      file if the route is ergonomic) that exercises the same sync from
      a `HudPlugin`-registered system path.
- [ ] **RESOLUTION dim overlay renders only while
      `Phase::Resolution`**: GIVEN a HUD test world in any visible
      phase, WHEN `Res<CurrentClientPhase>` is set to `Phase::Resolution`
      and `sync_dim_overlay_for_resolution_system` runs in the
      `PresentationSet::StateSync` schedule, THEN the `HudDimOverlay`
      root entity has `Visibility::Visible`; AND WHEN
      `Res<CurrentClientPhase>` is set to any other phase
      (DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT, GAME_OVER,
      LOBBY), THEN the `HudDimOverlay` root entity has
      `Visibility::Hidden`. *Evidence target*:
      `tests/integration/hud/hud_resolution_dim_test.rs` —
      `test_dim_overlay_visible_only_in_resolution` sub-test enumerates
      all `Phase` variants and asserts the visibility flip.
- [ ] **Dim overlay is pre-pooled, not per-update spawned**: GIVEN the
      `HudDimOverlay` entity ID is captured immediately after HUD
      session entry, WHEN three `S2CPhaseChanged` messages drive
      transitions PLACEMENT → RESOLUTION → DRAFT_SHOP → RESOLUTION,
      THEN the captured `HudDimOverlay` entity ID is unchanged across
      the four frames; no additional `HudDimOverlay` entity is spawned
      and no `HudDimOverlay` entity is despawned during phase
      transitions. *Evidence target*:
      `tests/integration/hud/hud_resolution_dim_test.rs` —
      `test_dim_overlay_pre_pooled` sub-test captures the entity ID,
      drives the transitions via the existing single
      `S2CPhaseChanged` drain, and asserts entity-ID stability.
- [ ] **Single source of phase truth preserved (TR-HUD-006 +
      ADR-002)**: GIVEN the HUD plugin registration, WHEN the union of
      systems registered by `HudPlugin` is enumerated, THEN exactly one
      system reads `MessageReceiver<S2CPhaseChanged>` (the existing
      `phase_sink_system` populating `Res<CurrentClientPhase>`); AND no
      system added by this story writes to `CurrentClientPhase` or
      emits a synthetic `S2CPhaseChanged`. *Evidence target*:
      `tests/integration/hud/hud_resolution_dim_test.rs` —
      `test_no_client_side_phase_authority` sub-test inspects the
      compiled system set and asserts the single-drain invariant via
      `App::world().resource::<Schedules>()` introspection or by
      direct grep evidence cited in the test doc-comment.
- [ ] **FROZEN-mode tiebreak preserved (TR-HUD-009 + ADR-011)**: GIVEN
      HUD is in `Phase::Resolution` with `HudDimOverlay` visible, WHEN
      `S2CPhaseChanged(GAME_OVER)` arrives and the HUD enters FROZEN,
      THEN the `HudDimOverlay` flips to `Visibility::Hidden` (FROZEN is
      not RESOLUTION); AND WHEN a subsequent `S2CGameSnapshot` arrives
      whose `phase == Resolution`, THEN the rebuild restores
      `HudDimOverlay` to `Visibility::Visible` and FROZEN re-applies
      after the rebuild settles. *Evidence target*:
      `tests/integration/hud/hud_resolution_dim_test.rs` —
      `test_frozen_mode_tiebreak_dim_overlay` sub-test mirrors the
      pattern used by HUD-19 and TR-HUD-009 in
      `tests/integration/hud/reconnect_snapshot_rebuild_test.rs`.
- [ ] **No countdown numerals on the timer bar (HUD-11
      preserved)**: GIVEN the wired timer bar is visible, WHEN the
      `HudTimerBar` entity and its descendants are inspected, THEN no
      `Text` or `TextSpan` component holds a countdown or elapsed-time
      value. *Evidence target*:
      `tests/integration/hud/hud_resolution_dim_test.rs` —
      `test_timer_bar_no_countdown_numerals` sub-test queries
      `Children` of `HudTimerBar` and asserts no `Text` or `TextSpan`
      child exists with a numeric countdown value.
- [ ] **Pre-pooled entity count incremented to 22**: GIVEN
      `HUD_ENTITY_COUNT = 22` after this story's overlay addition, WHEN
      `tests/integration/presentation/asset_wiring_foundation_test.rs`
      runs, THEN the assertion against `HUD_ENTITY_COUNT` passes; the
      foundation test file is updated in this story's diff to reflect
      the new count, mirroring PAW-004-f's 19→21 pattern. *Evidence
      target*: `tests/integration/presentation/asset_wiring_foundation_test.rs`
      diff in this story's commit set.
- [ ] **Manual evidence document captured**: GIVEN the implementation
      lands, WHEN
      `production/qa/evidence/sprint-10-hud-chrome-evidence.md` is
      authored, THEN it records the friend-game route, the build/commit
      SHA, the four screenshot captures (timer bar visible during
      timed phase; figurines showing class-specific art; dim overlay
      visible during RESOLUTION; dim overlay hidden during GAME_OVER
      FROZEN), and the explicit no-claim language: "no Standard-tier
      accessibility completion is claimed; QA-COND-0005 remains
      accepted-risk friend-game scope; no client-side optimistic phase
      authority added; existing `S2CPhaseChanged` drain remains the
      single source of phase truth." *Evidence target*: file at
      `production/qa/evidence/sprint-10-hud-chrome-evidence.md` (new).

---

## Implementation Notes

- HUD pool init lives in `client/src/ui/hud/mod.rs`. Add the
  `HudDimOverlay` marker entity to the same `spawn_hud_root_system`-style
  init system that already spawns the figurines, timer bar, and dot
  pool per PAW-004. The overlay is a full-viewport `Node` with
  `Visibility::Hidden` at session entry.
- Visibility control system: `sync_dim_overlay_for_resolution_system`.
  Reads `Res<CurrentClientPhase>`. Writes only `Visibility` on the
  `HudDimOverlay` query. Registered in `HudPlugin::build` in the
  `PresentationSet::StateSync` schedule (per ADR-021 — same set the
  scoreboard dot Observer transitions live in).
- Do not animate the overlay. Visibility flips are instantaneous per
  HUD-12b BLOCKING. No `Animator<T>` is attached. No
  `Animator::set_tweenable()` call is added.
- The wired timer bar handle is already set at PAW-004 init. If the
  friend-game route shows the placeholder fallback instead, the most
  likely cause is a missed `HudPlugin` registration in `PresentationPlugin`
  or a startup ordering bug between asset load and HUD pool init —
  trace via `client/src/asset_wiring.rs` and the `LoadingState`
  exit point. Do not add a new `add_systems` call outside `HudPlugin`.
- The class figurine sync (`sync_figurine_image_system`) already exists
  per PAW-004-d. If the live route shows placeholder art, verify the
  system is registered and that
  `PresentationGameSnapshotMessage` arrives with non-default class
  fields on the friend-game route. Add no new sync system.
- The dim overlay's `BackgroundColor` alpha is a tuning value; default
  to a value that visibly dims the underlying HUD without obscuring
  gold/mana/phase readouts. Record the chosen alpha in the evidence
  document so a future polish pass can revisit it without re-deriving
  the constant.
- The `HUD_ENTITY_COUNT` constant lives in
  `client/src/ui/hud/mod.rs` (or wherever PAW-004-f set it to 21).
  Bump it to 22 and update the foundation test in the same commit.
- Use the `liv-bevy-018` skill for any Bevy 0.18 API question. Bevy
  0.18 Required Components mean `Node { ... }` is spawned directly;
  `NodeBundle` is removed. `Visibility::Visible` / `Visibility::Hidden`
  toggling on a parent propagates via the existing
  `bevy_render::view::visibility` propagation system.
- For the test file pattern, mirror
  `tests/integration/hud/reconnect_snapshot_rebuild_test.rs` (already
  on `main` from S10-TD-001 Wave E `bb51463`) which establishes the
  HUD partial-App fixture pattern with `init_state::<ClientState>()`,
  `AssetPlugin::default()`, `init_asset::<Image>()`, and
  `placeholder_assets_for_tests()`.

---

## Performance Budget

Steady-state: zero added cost — the overlay is a pre-pooled `Node`
toggled via `Visibility`, not spawned/despawned per frame. Phase
boundary spikes: well under 1 ms — a single `Visibility` write on a
single entity per phase transition. No per-frame entity creation, no
per-frame texture upload, no per-frame text layout. Conforms to the
control-manifest guardrail "Presentation steady-state work stays below
1 ms/frame; phase-boundary presentation spikes stay below 3 ms."

---

## QA Test Cases

(Source: `production/sprints/sprint-10.md:95` AC text; mirrored into
the evidence document at sign-off.)

- **Timer bar wired in timed phases**
  - Given: friend-game session in DRAFT_INITIAL, DRAFT_SHOP,
    DRAFT_AUCTION, or PLACEMENT.
  - When: HUD renders.
  - Then: `HudTimerBar.ImageNode.image` resolves to
    `HUD_PHASE_TIMER_BAR_ASSET` (not the placeholder fallback).

- **Class figurines wired**
  - Given: `PresentationGameSnapshotMessage { own_class, opponent_class
    }` arrives with non-default classes.
  - When: HUD state sync completes.
  - Then: own and opponent `HudFigurine.ImageNode.image` handles each
    resolve to the class-specific asset constant.

- **Dim overlay during RESOLUTION**
  - Given: HUD in `Phase::Placement`, `HudDimOverlay` hidden.
  - When: `S2CPhaseChanged(RESOLUTION)` arrives via the existing
    single `phase_sink_system` drain and
    `sync_dim_overlay_for_resolution_system` runs.
  - Then: `HudDimOverlay` is `Visibility::Visible`.

- **Dim overlay lifts on phase exit (non-FROZEN path)**
  - Given: HUD in `Phase::Resolution`, `HudDimOverlay` visible.
  - When: `S2CPhaseChanged(DRAFT_SHOP)` arrives and the next
    `PresentationSet::StateSync` tick runs.
  - Then: `HudDimOverlay` is `Visibility::Hidden`.

- **Dim overlay hidden during GAME_OVER FROZEN (TR-HUD-009)**
  - Given: HUD in `Phase::Resolution`, `HudDimOverlay` visible.
  - When: `S2CPhaseChanged(GAME_OVER)` arrives.
  - Then: `HudMode == FROZEN`; `HudDimOverlay.Visibility::Hidden`;
    no subsequent `S2CGoldUpdate` or `HudObjectiveUpdate` mutates the
    HUD per HUD-10.

- **Snapshot rebuild restores overlay state (ADR-011)**
  - Given: HUD just received `S2CPhaseChanged(GAME_OVER)` after a
    RESOLUTION phase; FROZEN is engaged.
  - When: a late `S2CGameSnapshot { phase: Resolution, ... }` arrives.
  - Then: HUD performs a snapshot-wins rebuild; `HudDimOverlay`
    restores to `Visibility::Visible`; FROZEN re-applies after rebuild
    completes.

- **No countdown numerals (HUD-11)**
  - Given: timer bar visible.
  - When: timer bar entity descendants are queried.
  - Then: no `Text` or `TextSpan` child holds a numeric countdown
    value.

- **No client-side phase authority (TR-HUD-006 + ADR-002)**
  - Given: `HudPlugin` registration.
  - When: the union of registered systems is inspected.
  - Then: exactly one system reads `MessageReceiver<S2CPhaseChanged>`
    (the existing `phase_sink_system`); no system writes
    `CurrentClientPhase` or emits a synthetic `S2CPhaseChanged`.

- **Manual screenshot capture**
  - Given: a built friend-game client and a two-client local route.
  - When: a manual play-through reaches each phase.
  - Then: the four required screenshots are captured and recorded in
    `production/qa/evidence/sprint-10-hud-chrome-evidence.md` with
    build/commit SHA, browser/viewport, and the explicit no-claim
    language.

---

## Test Evidence

**Story Type**: Integration

**Required automated test target**:

- `tests/integration/hud/hud_resolution_dim_test.rs` (new)
- `cargo test -p client --test hud_resolution_dim_test`

**Required regression targets** (must remain green):

- `cargo test -p client --test hud_reconnect_snapshot_rebuild_test`
- `cargo test -p client --test hud_same_tick_tie_break_test`
- `cargo test -p client --test hud_scoreboard_dot_message_test`
- `cargo test -p client --test hud_text_size_contrast_accessibility_test`
- `cargo test -p client --test hud_phase_label_round_counter_test`
- `cargo test -p client --test hud_phase_transitions_test`
- `cargo test -p client --test hud_game_over_freeze_test`
- `cargo test -p client --test hud_gold_mana_display_test`
- `cargo test -p client --test hud_economy_auction_inline_gold_test`
- `cargo test -p client --test hud_numeric_tween_animation_test`
- `cargo test -p client --test hud_mana_shape_distinction_test`
- `cargo test -p client --test asset_wiring_foundation_test`
- `cargo test -p client --test hud_asset_wiring_test`

**Required foundation test update**:

- `tests/integration/presentation/asset_wiring_foundation_test.rs` —
  bump `HUD_ENTITY_COUNT` assertion 21 → 22 (mirrors PAW-004-f).

**Required manual evidence path**:

- `production/qa/evidence/sprint-10-hud-chrome-evidence.md` (new) —
  records build/commit SHA, browser/viewport, four phase captures,
  chosen `BackgroundColor` alpha for the dim overlay, and the explicit
  no-claim language listed in AC9.

**Status**: [ ] Not yet implemented — story authored as a Sprint 10
docs-only prerequisite per `production/sprints/sprint-10.md` lines
117–135.

---

## Files Modified

Anticipated diff (final shape may vary; this is the authoring-time
target):

| Path | Change |
|---|---|
| `client/src/ui/hud/mod.rs` | Add `HudDimOverlay` marker; spawn dim overlay at HUD pool init; register `sync_dim_overlay_for_resolution_system` in `PresentationSet::StateSync`; bump `HUD_ENTITY_COUNT` 21 → 22. |
| `tests/integration/hud/hud_resolution_dim_test.rs` | NEW — 8 sub-tests per the AC list. |
| `tests/integration/presentation/asset_wiring_foundation_test.rs` | Bump asserted `HUD_ENTITY_COUNT` 21 → 22. |
| `client/Cargo.toml` | Register `[[test]] name = "hud_resolution_dim_test"` entry. |
| `production/qa/evidence/sprint-10-hud-chrome-evidence.md` | NEW — manual evidence document per AC9. |

No protocol files, no `shared/` files, no `server/` files, no
`design/gdd/`, no `docs/architecture/` files are modified. No new asset
files are added.

---

## Dependencies

- Depends on: `production/epics/presentation-asset-wiring/story-004-hud-figurines-timer-dots.md`
  (PAW-004) — Done. Provides the `HudFigurine`, `HudTimerBar`, and
  objective dot `ImageNode` entities this story consumes.
  Verification: `git log --oneline a7e397a 2132129` returns both
  commits on `main`.
- Depends on: S10-PAW-001 PAW-004 row reaching `done` in
  `production/sprint-status.yaml` (the formal `/story-done` for
  PAW-004 — substantive code already on `main`).
- Depends on: `client/src/asset_wiring.rs` `HUD_PHASE_TIMER_BAR_ASSET`
  and class figurine path constants — present on `main` since
  `a7e397a`.
- Depends on: `client/src/ui/hud/mod.rs` `phase_sink_system` (existing
  single `S2CPhaseChanged` drain populating
  `Res<CurrentClientPhase>`) — present on `main`.
- Depends on: `tests/integration/hud/reconnect_snapshot_rebuild_test.rs`
  fixture pattern — present on `main` since `bb51463` (S10-TD-001
  Wave E).
- Depends on: ADR-021, ADR-011, ADR-002, ADR-001, ADR-008 all
  Accepted (per
  `docs/architecture/control-manifest.md` header line 6 — control
  manifest covers all 22 ADRs).
- Unlocks: friend-game-route visible chrome MVP for the HUD
  (`/story-done` flips `production/sprint-status.yaml`
  S10-POLISH-001 → `done`).

---

## Readiness Notes

**Implementation readiness verdict at authoring time**: READY pending
`/story-readiness` re-run.

- Story file authored at the canonical Sprint 10 path agreed in PROMPT
  614 follow-up: `story-013-` slot used because `story-011-` and
  `story-012-` are already occupied in the HUD epic by
  `story-011-current-reserve-mana-shapes.md` (Sprint 6 / A11Y-ST-13)
  and `story-012-text-size-and-contrast-accessibility.md` (Sprint 6
  accessibility). The Sprint 10 plan's row 127 path is updated in the
  same commit to reference this corrected file.
- All TR-IDs, ADR refs, control-manifest version, engine notes, test
  evidence path, out-of-scope, and dependency rows are embedded
  per the PROMPT 614 readiness review gap list.
- Slot collision rationale is recorded in the commit body so the
  next reader can trace why row 127 references `story-013-` rather
  than `story-011-`.

---

## Definition of Done

This story is **ready to start** at authoring time, not yet
substantively complete. Done means:

- All acceptance criteria above checked.
- Automated test
  `tests/integration/hud/hud_resolution_dim_test.rs` passing under
  `cargo test -p client --test hud_resolution_dim_test`.
- Foundation test updated and passing
  (`HUD_ENTITY_COUNT == 22`).
- All listed regression targets remain green.
- Manual evidence document at
  `production/qa/evidence/sprint-10-hud-chrome-evidence.md` exists,
  records the four required captures with build/commit SHA, the
  chosen overlay alpha, and the explicit no-claim language.
- `/story-done` flips `production/sprint-status.yaml`
  S10-POLISH-001 → `done`.
- No public-release readiness, full playable-client manual QA, full
  game completion, or broad Standard-tier accessibility completion is
  claimed at close.
