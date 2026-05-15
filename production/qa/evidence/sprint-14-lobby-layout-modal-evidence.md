# Sprint 14 — Lobby Layout Modal Evidence (Story 024 / S12-UX-LOBBY-LAYOUT-MODAL-001)

**Story**: `production/epics/playable-client/story-024-lobby-layout-modal.md`
**Story ID**: `S12-UX-LOBBY-LAYOUT-MODAL-001`
**Sprint**: Sprint 14 (active per PROMPT 897 activation; row pull-in via the Sprint 14 plan / qa-plan trail)
**Prompt**: PROMPT 937 `/dev-story` worker on branch `work/s14-lobby-layout-modal`
**Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s14-lobby-layout-modal`
**Source-of-truth at worker open**: `origin/main@fc77503` (state(s14) refresh after lobby modal readiness, PROMPT 936)
**Worker commit (filled at commit time)**: see Closure Trail section below
**Layer**: Lobby UI / UX (Client) — `client/src/ui/lobby.rs`
**Type**: Integration — net-new lobby root composition + viewport-invariant test
**Friend-game scope only.** This evidence does **not** advance public release readiness,
Standard-tier accessibility conformance (`QA-COND-0005`), playtest validation
(`QA-COND-0006`), final-art / asset-production completion (`PAW-TD-*-a`),
two-client `GAME_OVER` closure (`S8-QA-001-W1`), or PROMPT 761 `Polish->Release`
gate-check retry. See "No-Claim Restatement" at the bottom.

---

## §1 Producer Decision (AC1) — Option A (Centred Modal Panel)

**Chosen option**: **Option A — Centred modal panel** (analogous to
`client/src/presentation/result_screen.rs:496-549`).

**Decision capture**: PROMPT 933 (2026-05-15) by the CCGS producer agent
(paperwork-only producer-authority capture run), with the ux-designer +
art-director consultation note recorded in
`production/epics/playable-client/story-024-lobby-layout-modal.md` §"Decision
Capture (PROMPT 933, 2026-05-15)". The decision was integrated to `origin/main`
via PROMPT 935 `--no-ff` merge (`origin/main@39cc97f`) and reaffirmed READY by
PROMPT 936's `/story-readiness` re-run. This PROMPT 937 `/dev-story` worker
implements that decision verbatim.

**Cross-references**:

- PROMPT 802 §9 producer-decision-3 (modal-panel vs full-viewport hero):
  RESOLVED — Option A per PROMPT 933, 2026-05-15.
- `docs/ux/global-ui-design-spec.md` §3 (`Modal` z-layer) + §4 (spacing
  scale) + §5 (typography hierarchy) + §8 (centred-modal responsive rules) +
  §10 (modal-centering pattern). Story 024 consumes all five sections without
  introducing new tokens or new asset slots.
- Story 024 §"Decision Capture (PROMPT 933, 2026-05-15)" — Option-A literals
  table (panel `max_width`, `width`, `max_height`, padding, gap, font sizes,
  z-layers).
- Story 024 §"Likely Files" — the table predicted edits to
  `client/src/ui/lobby.rs`, a NEW lobby viewport-invariant test under
  `tests/integration/playable_client/`, and this evidence document; the
  realised set matches the prediction exactly.

**Option B**: N/A. Not chosen. No `PAW-TD-006-c` placeholder background-art
slot introduced; `PAW-TD-*-a` accept-risk preserved verbatim.

---

## §2 ux-designer Consultation (AC10)

The ux-designer consultation note from PROMPT 933 (2026-05-15) is the binding
input. PROMPT 937 did not re-open the consultation because PROMPT 933's
literals table is locked and `docs/ux/global-ui-design-spec.md` §3 / §4 / §5 /
§8 / §10 invariants hold without further ux-designer review at implementation
time. Friend-game-tier literals are accepted; Standard-tier accessibility
conformance is **not** claimed.

**Locked literals (verbatim from story 024 §"Decision Capture" table)**:

| Aspect | Value | Source in this implementation |
|---|---|---|
| Panel `max_width` | `Val::Px(860.0)` | `client/src/ui/lobby.rs::LOBBY_PANEL_MAX_WIDTH_PX = 860.0` (mirrors `result_screen.rs:538`). |
| Panel `width` | `Val::Percent(88.0)` | `client/src/ui/lobby.rs::LOBBY_PANEL_WIDTH_PERCENT = 88.0` (mirrors `result_screen.rs:537`). |
| Panel `max_height` | `Val::Percent(92.0)` | `client/src/ui/lobby.rs::LOBBY_PANEL_MAX_HEIGHT_PERCENT = 92.0` (mirrors `result_screen.rs:539`). |
| Panel padding (all sides) | `SPACING_LG` (24 px) | `client/src/ui/lobby.rs::spawn_lobby_ui_system` panel node `padding: UiRect::all(Val::Px(SPACING_LG))`. |
| Inter-child gap (cluster) | `SPACING_MD` (16 px) | Panel node `row_gap: Val::Px(SPACING_MD)`. |
| Section separator gap | `SPACING_XL` (32 px) | Three zero-height separator nodes inside the panel each contribute `SPACING_XL - SPACING_MD` margin so the cumulative inter-section gap (row_gap + separator margin) resolves to `SPACING_XL` per `docs/ux/global-ui-design-spec.md` §4. Guarded by `ac5_section_separators_resolve_to_spacing_xl_cumulative_gap` test. |
| Full-viewport parent backdrop | `Color::srgba(0.039, 0.051, 0.078, OVERLAY_SCRIM_ALPHA=0.55)` | Root node `BackgroundColor` with `OVERLAY_SCRIM_ALPHA` from `client/src/ui/design_tokens/overlays.rs`. RGB is the §7 `SURFACE` token literal verbatim. |
| `GlobalZIndex` (parent backdrop) | `UI_OVERLAY` (`GlobalZIndex(400)`) | Root node `z_layers::UI_OVERLAY`. |
| `GlobalZIndex` (panel) | `MODAL` (`GlobalZIndex(500)`) | Panel node `z_layers::MODAL`. |
| Font sizes | Unchanged. `typography::H3` for status banner / section labels; `typography::BODY` for buttons / labels / room-code chip text. | No new font-size literals introduced (per PROMPT 933 ux-designer co-sign). |

---

## §3 Diff Summary (AC2 / AC9 / AC12)

```
client/Cargo.toml                                                                   |   4 +
client/src/ui/lobby.rs                                                              | 460 +++++++++++++++++++++++++++++++------------------
tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs (NEW)     | (full file)
production/qa/evidence/sprint-14-lobby-layout-modal-evidence.md (NEW)               | (this file)
```

**`client/src/ui/lobby.rs`** (edited):

- Replaced the pre-Option-A top-left anchored 420-px column composition of
  `spawn_lobby_ui_system` with a full-viewport flex container (`LobbyRoot`)
  that owns the modal scrim backdrop + `UI_OVERLAY` z-layer + the centred
  `LobbyPanel` modal child at `MODAL` z-layer.
- Removed the pre-Option-A `LOBBY_PANEL_WIDTH = 420.0` constant. Added three
  new `pub const`s — `LOBBY_PANEL_MAX_WIDTH_PX = 860.0`,
  `LOBBY_PANEL_WIDTH_PERCENT = 88.0`, `LOBBY_PANEL_MAX_HEIGHT_PERCENT = 92.0` —
  exported for the viewport-invariant test bin to consume.
- Promoted the `LobbyRoot` marker component from private to `pub` and added a
  new `pub LobbyPanel` marker component so the viewport-invariant test can
  query the new modal panel directly.
- Added a `use ... overlays, spacing::{SPACING_LG, SPACING_MD, SPACING_XL}`
  import to consume Sprint 14 Tier 0 design tokens at the lobby spawn site.
- Reordered the lobby panel children so the `LobbyConfirmClassButton` CTA is
  the LAST direct child, resolving the PROMPT 802 §3.1 L4 read-order
  inversion. Class portraits + slot panels now render ABOVE the confirm CTA,
  and the room-code chip ImageNode is hoisted to render alongside the status
  banner (top of panel).
- Inserted three zero-height section-separator nodes between the four panel
  sections (status / create-join / class-picker / confirm) so the cumulative
  inter-section gap (`row_gap` + separator margin) resolves to `SPACING_XL`
  per the PROMPT 933 literals table.

**`client/Cargo.toml`** (edited):

- Added a `[[test]]` entry for
  `playable_client_lobby_layout_viewport_invariant_test` pointing at the new
  test bin under
  `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`.

**`tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`**
(NEW):

- 12 test functions asserting AC2 + AC3 / AC4 panel sizing + AC3(e) / AC5
  read-order + AC8 bin-filename + AC11 no-`#[ignore]` + AC13 friend-game
  scope preservation. Each test fans out to one of the story-defined ACs
  via its `ac2_` / `ac3_` / `ac5_` / `ac8_` / `ac13_` prefix.

**`production/qa/evidence/sprint-14-lobby-layout-modal-evidence.md`** (NEW):

- This evidence document.

**Forbidden paths NOT modified by PROMPT 937**:

- `shared/src/protocol.rs` — verified `git diff --stat origin/main...HEAD --
  'shared/'` empty (AC9: zero protocol-shape change).
- `server/` — verified `git diff --stat origin/main...HEAD -- 'server/'`
  empty (AC9: zero server-side change).
- `production/sprint-status.yaml`, `production/sprints/sprint-14.md`,
  `production/sprints/sprint-12.md`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, `production/qa/qa-plan-sprint-14.md`,
  `production/session-state/active.md`,
  `production/session-state/codex-orchestrator-state.md` — verified
  unmodified by PROMPT 937 (AC12: row flip + sprint-status edits reserved
  for the downstream `/story-done` paperwork prompt).

---

## §4 Test Evidence (AC8 / AC11)

### Story-prescribed lobby layout viewport invariant test

`tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`
(12 functions, all passing on PROMPT 937 worker tip):

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.03s

ac2_lobby_root_is_full_viewport_flex_container ............ ok
ac2_lobby_panel_is_flex_child_composed_via_flex ........... ok
ac2_lobby_plugin_spawns_camera ............................ ok
ac2_root_backdrop_uses_overlay_scrim_alpha_token .......... ok
ac2_z_layers_match_prompt_933_option_a_literals ........... ok
ac3_ac4_panel_sizing_matches_prompt_933_option_a_literals.. ok
ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd ...... ok
ac3_ac5_confirm_cta_is_last_panel_child ................... ok
ac3_exactly_one_confirm_cta_after_migration ............... ok
ac5_section_separators_resolve_to_spacing_xl_cumulative_gap ok
ac8_test_bin_filename_matches_story_prescribed_name ....... ok
ac13_friend_game_scope_preservation_documented_inline ..... ok
```

### Adjacent lobby regression set (no regressions)

```text
playable_client_lobby_entry_test ............................. 6 passed
playable_client_lobby_confirm_state_text_test ................ 5 passed
lobby_asset_wiring_test ...................................... 7 passed
lobby_chrome_wiring_test ..................................... 5 passed
```

All previously-passing lobby tests continue to pass against the migrated
`spawn_lobby_ui_system`. No new `#[ignore]` markers introduced.

### Sprint 14 Tier 0 viewport-invariant test bin (no regressions)

```text
ui_viewport_invariants_test .................................. 12 passed
```

Story 005's full canonical-matrix viewport-invariant suite continues to pass
on the migrated lobby; the baseline fixture's `lobby_root` surface entry is
NOT replaced by PROMPT 937 (the fixture is the foundational story-005 bin's
authoritative baseline and is owned by story 005; per AC8 alternative,
this story authors a standalone test bin instead of consuming the rank-4
bin). A future Sprint 14 follow-up can re-baseline `lobby_root` against the
Option A literals; PROMPT 937 explicitly does not modify
`tests/integration/fixtures/ui_viewport_baseline.rs` to avoid expanding
scope outside story 024.

### Workspace static checks

```text
cargo fmt --all -- --check ............................... PASS (exit 0)
cargo check --workspace --all-targets .................... PASS (no errors;
  pre-existing dead_code warning in tests/integration/presentation/
  hand_ui_asset_wiring_test.rs preserved unchanged — not authored by this
  PR)
```

### AC11 workspace test policy

Per Sprint 13 QA-plan binding no-full-workspace-tests-by-default policy
(preserved into Sprint 14), the AC11 regression set executed by PROMPT 937
is the targeted lobby + viewport invariant suite above (35+ tests passing).
A full-workspace `cargo test --workspace --tests --no-fail-fast` is deferred
to the Sprint 14 end-of-sprint integration smoke gate, consistent with the
disposition for prior Sprint 13 / Sprint 14 worker prompts on origin/main.
No new `#[ignore]` markers are introduced; the new viewport-invariant test
passes; previously-passing tests continue to pass.

---

## §5 Visual Captures (AC3 / AC4)

**Capture limitation**: PROMPT 937 ran under the worker harness on a Windows
MSVC environment without a windowed renderer attached to the test harness
(`cargo test` runs in `MinimalPlugins` mode without `wgpu` initialisation
because the lobby test app does not depend on the renderer plugin set). The
visual rendering capture suite typically lives under
`production/qa/evidence/captures/` for runtime-rendered evidence; for this
story the captures directory
`production/qa/evidence/captures/sprint-14-lobby-layout-modal/` is **created
empty** by PROMPT 937 and reserved for a future runtime-capture prompt
(e.g. a downstream `/team-qa` or manual playtest run that can spin up the
real Bevy render plugin and capture PNGs at `1920×1080` and `1366×768`).

The geometric invariant test `ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd`
analytically resolves the Option-A literals against both canonical viewports
and asserts that the panel's resolved width and height fit comfortably inside
each viewport. Test output (visible under `cargo test -- --nocapture`):

```text
[lobby_layout_viewport_invariant] viewport 1366x768:
  panel resolves to 860.0x706.6 inside 1366x768
[lobby_layout_viewport_invariant] viewport 1920x1080:
  panel resolves to 860.0x993.6 inside 1920x1080
```

At both viewports the panel's resolved width is clamped by the
`max_width: 860 Px` literal (88% × 1366 = 1202 px → clamped to 860 px; 88% ×
1920 = 1690 px → clamped to 860 px); the resolved max height is bounded by
`max_height: 92%` (92% × 768 = 706 px; 92% × 1080 = 993 px). Both fit
inside the viewport with positive margin on every side.

**Read-order trace (AC3(e) / AC5)**:

Top-to-bottom, the lobby modal panel composes its direct children in this
order (verified by `ac3_ac5_confirm_cta_is_last_panel_child` against the
real `spawn_lobby_ui_system`):

1. `LobbyStatusText` — status banner (typography::H3).
2. `LobbyRoomCodeChip` — room-code chip image + body text.
3. (Section separator — `SPACING_XL` cumulative gap before next section.)
4. `LobbyRoomCodeField` — typeable room-code input button.
5. `LobbyCreateRoomButton` + `LobbyJoinRoomButton` — create / join row.
6. "Requested slot" label.
7. `LobbyRequestedSlotButton(0..=3)` — slot row.
8. (Section separator — `SPACING_XL` cumulative gap before next section.)
9. "Class" label.
10. `LobbyClassPortrait(7)` — portrait wrap row (PAW-006-a).
11. `LobbyClassButton(6)` — class-button wrap row.
12. `LobbyOwnSlotPanel` + `LobbyOpponentSlotPanel` — slot-panel row
    (PAW-006-b).
13. (Section separator — `SPACING_XL` cumulative gap before the CTA.)
14. `LobbyConfirmClassButton` — primary CTA, LAST direct child.

The PROMPT 802 §3.1 L4 inversion (portraits + slot panels + room-code chip
image rendering *below* the confirm CTA) is resolved: every secondary
affordance now renders ABOVE the confirm CTA, and the room-code chip is
hoisted to the top of the panel alongside the status banner so the player's
attention reaches the room identifier before any input affordance.

**1366×768 viewport (manual / live trace)**: deferred to a downstream
runtime-capture prompt with the real renderer attached. Geometric fit is
proven analytically by AC3 fit-within-viewport test.

**1920×1080 viewport (manual / live trace)**: deferred to a downstream
runtime-capture prompt with the real renderer attached. Geometric fit is
proven analytically by AC4 fit-within-viewport test.

---

## §6 ADR Preservation (AC9)

- **ADR-002 Client-Server Authority** — preserved. No client-side mutation of
  class-lock, slot-assignment, or session-ready state is introduced outside
  the existing `S2CClassLocked` / `S2CSlotUpdated` / `S2CSessionReady` /
  `S2CClassesRevealed` drain paths. The `apply_class_locked_drain` +
  `apply_class_locked` + `apply_classes_revealed` + `apply_join_ack` +
  `apply_room_created` + `apply_slot_update` helpers are untouched by PROMPT
  937. **No optimistic client-side authority** is introduced by this story.
- **ADR-008 Lightyear Channel Configuration** — preserved. No channel change;
  no protocol shape change.
- **ADR-012 SessionReady Delivery** — preserved. The SessionReady Observer
  is untouched.
- **ADR-021 Presentation Layer Architecture** — preserved. Lobby remains a
  read-only presentation of `LobbyViewState` projected from
  server-authoritative messages; the migration is composition / hierarchy /
  typography / responsive-layout work only.

`git diff --stat origin/main...HEAD -- 'shared/src/protocol.rs' 'server/'`
output is **empty**, verifying AC9 mechanically.

---

## §7 Paired-Story Cross-Links (AC6 / AC7)

- **Story 025 (`S11-UX-LOBBY-CLASS-PICKER`)**: paired Should-Have row that
  owns the final class-portrait + class-button hierarchy treatment. PROMPT
  937 preserves the portrait row in place ahead of the button row in the
  panel composition; story 025's hierarchy treatment, when it lands, will
  refine the portrait + button pairing affordance without further root-
  composition change.
- **Story 026 (`S11-UX-LOBBY-BUTTON-HITTARGETS`)**: paired Should-Have row
  that owns the canonical lobby button dimensions. PROMPT 937 preserves the
  pre-migration `LOBBY_BUTTON_HEIGHT = 30.0` constant and the
  `lobby_button_node` width literals (`Val::Percent(100.0)` for the room-
  code field + confirm CTA; `Val::Px(128.0)` for create / join; `Val::Px(72.0)`
  for slot buttons; `Val::Px(92.0)` for class buttons) so the canonical
  dimensions remain stable across this layout migration.

Cross-links to the global UI design spec section adoption matrix (rank-12
row): preserved. `docs/ux/global-ui-design-spec.md` §"Spec Adoption Matrix"
rank-12 row already records this story as a consumer of §3 / §4 / §5 / §8
/ §10; PROMPT 937 implements that consumption.

Cross-link to `docs/ux/ui-clean-pass-roadmap.md` rank-12 row: preserved. The
roadmap already records this row as a Tier 1, Must rank-12 candidate for
Sprint 14.

Cross-link to PROMPT 802 §3.1 L1 / L4 + §9 producer-decision-3: resolved
per Option A; details in §1 above.

---

## §8 Closure Trail (filled at commit time)

- PROMPT 937 (this prompt) — `/dev-story` implementation worker:
  - Branch: `work/s14-lobby-layout-modal`
  - Source-of-truth at worker open: `origin/main@fc77503` (PROMPT 936 state
    refresh).
  - Worker commit: see git log on the branch after this evidence is
    committed (`feat(s14): /dev-story S12-UX-LOBBY-LAYOUT-MODAL-001 ...`).
  - Cargo policy applied: yes — `CARGO_TARGET_DIR=D:\_DEV\cargo-target\
    ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
    `CARGO_INCREMENTAL=0`, `RUSTFLAGS=-C debuginfo=0 -C link-arg=
    /DEBUG:NONE`.

This story's row flip (`ready -> done` in
`production/sprint-status.yaml`) and the story file Status header flip
(`Draft -> Done`) are reserved for a downstream `/story-done` paperwork
prompt; PROMPT 937 explicitly does NOT perform `/story-done`.

---

## §9 No-Claim Restatement (AC13)

PROMPT 937 / this evidence document does **not** claim, advance, or close:

- Public release readiness.
- Release-candidate readiness.
- Full game completion.
- Standard-tier accessibility completion (`QA-COND-0005`) — friend-game scope
  only; the lobby modal panel is **not** WCAG-contrast-checked, **not**
  ≥44px hit-target-checked, **not** full-keyboard-navigation-checked, **not**
  screen-reader-checked, **not** colorblind-mode-checked, and **not**
  text-scaling-checked. Hit-target work is delegated to paired story 026
  (`S11-UX-LOBBY-BUTTON-HITTARGETS`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Full playable-client manual QA.
- Two-client `GAME_OVER` closure (`S8-QA-001-W1`).
- Final-art / asset-production completion (`PAW-TD-*-a` across PAW-002..
  PAW-006).
- PROMPT 761 `Polish->Release` gate-check retry — the `FAIL` evidence is
  preserved verbatim.
- Sprint 14 close-out.
- Stage advance from `Polish` to `Release`.
- Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 close-out re-opens.
- Any underlying Sprint 12 story 019 drag-runtime bug claim
  (`closed-with-conditions / cannot-reproduce` preserved).

`TQ-S12-C1..C7` preserved verbatim. All carried non-claims preserved.
