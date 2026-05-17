# Sprint 15 / Story 008 — S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 — Evidence

**Story**: `production/epics/ui-clean-pass/story-008-ui-interaction-state-primitives.md`
**Spec amendment**: `docs/ux/global-ui-design-spec.md` §11 (NEW), §10
button-affordance forward references, Spec Adoption Matrix row updated
**Worker run**: PROMPT 1005 `/dev-story`
**Source-of-truth at worker base**:
`origin/main@84e621eac242ad333a9597d45b5cd43dd39f98cc` (PROMPT 1002
Sprint 15 QA-plan integration tip)
**Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/s15-ui-interaction-state-primitives-1005`
**Branch**: `work/s15-ui-interaction-state-primitives`

This evidence document mirrors the structure of
`production/qa/evidence/sprint-14-overlay-alpha-token/evidence.md`. AC1
through AC9 are covered by code + automated tests; AC10 (per-surface
migration OUT OF SCOPE) is verified by `git diff` against the lobby /
auction / HUD / presentation surfaces; AC11 (friend-game scope preserved)
is verified by `git diff` of `production/sprint-status.yaml`; AC12 (no
closure / release / playtest / final-art / GAME_OVER claims) is the
authoring discipline of the prompt itself. No optimistic client-side
authority is introduced. `PAW-TD-*-a` / `QA-COND-0005` / `QA-COND-0006`
accept-risk dispositions are unchanged. PROMPT 761 Polish→Release `FAIL`
preserved. `S8-QA-001-W1` OPEN.

---

## Cargo resource policy applied

Worker session set the binding Windows/MSVC Cargo resource policy before
every `cargo` invocation per the PROMPT 1005 brief:

```text
CARGO_TARGET_DIR        = D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG = 0
CARGO_PROFILE_TEST_DEBUG= 0
CARGO_INCREMENTAL       = 0
RUSTFLAGS               = -C debuginfo=0 -C link-arg=/DEBUG:NONE
```

Disk free at session start: D: 826 GB free / 1.3 TB total (above the
40 GB threshold; no stale target cleanup required).

---

## Files touched

| Path | Status | Change |
|------|--------|--------|
| `client/src/ui/design_tokens/interaction_states.rs` | NEW | Author HOVER_* (2 alpha tokens) + FOCUS_* (1 Color + 2 px tokens) + PRESSED_* (1 alpha + 1 px) + DISABLED_* (3 alpha tokens) constant families with `///` doc comments and inline `#[cfg(test)] mod tests` (per-family range, canonical-band, focus-color-ratification, visual-state-ordering, audit-array invariants). |
| `client/src/ui/design_tokens/mod.rs` | EDIT | Append `pub mod interaction_states;` re-export + module-doc bullet for `interaction_states`. |
| `docs/ux/global-ui-design-spec.md` | EDIT | (a) §2 Scope Boundaries: flip interaction-state primitives entry from "owned by ... not this spec" to "authored by ... and ratified in §11". (b) §10 Primary / Secondary button affordance: flip "Hover / pressed: future scope" deferral notes to "see §11 'Interaction State Primitives'" forward references. (c) Insert NEW `## §11 Interaction State Primitives` section enumerating Hover / Focus / Pressed / Disabled token tables, friend-game scope guard, visual-state ordering invariants, and Sprint 15 scope statement. (d) Spec Adoption Matrix row for `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` extended to cite §11 alongside §7 and §10. (e) Ratification scope guard updated to reflect §11 authoring while preserving deferred per-surface migration. |
| `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs` | NEW | AC1 / AC6 reach-through exports, AC2 / AC4 / AC5 audit-array unit-interval, AC3 / AC4 audit-array pixel-bounds, AC3 focus-color spec §7 ratification, AC4 / AC5 visual-state ordering, AC8 / AC9 doc-comment source-scan, AC9 module-prefix grep guard, AC7 spec §11 anchor scan. |
| `client/Cargo.toml` | EDIT | Register `[[test]] ui_clean_pass_interaction_state_primitives_test` entry. |
| `production/qa/evidence/sprint-15-ui-interaction-state-primitives/evidence.md` | NEW | This document. |

No write under `server/`, `shared/`, `production/sprint-status.yaml`,
`production/sprints/sprint-15.md`, `production/qa/qa-plan-sprint-15.md`,
`production/stage.txt`, or any `production/session-state/*` — verified
via `git diff --stat` against `origin/main` at commit time.

---

## AC verdicts

| AC | Verdict | Verification |
|----|---------|--------------|
| AC1 — Interaction-state primitive module authored | PASS | `client/src/ui/design_tokens/interaction_states.rs` NEW; `mod.rs` re-exports `pub mod interaction_states;`; integration test `ac1_ac6_module_exports_four_named_token_set_families` proves the four named families import successfully. |
| AC2 — Named hover tokens with defaults | PASS | `HOVER_BG_TINT_ALPHA: f32 = 0.08` (band `0.04..=0.16`), `HOVER_BORDER_ALPHA: f32 = 0.40` (band `0.20..=0.60`); inline test `ac2_hover_tokens_in_documented_alpha_range` + integration test `ac2_ac4_ac5_audit_array_alphas_in_unit_interval` enforce ranges. |
| AC3 — Named focus tokens with defaults | PASS | `FOCUS_RING_COLOR: Color = Color::srgb(0.949, 0.788, 0.298)` (verbatim §7 `ACCENT` triple, hex `#F2C94C`); `FOCUS_RING_WIDTH_PX: f32 = 2.0` (band `1.0..=3.0`); `FOCUS_RING_OFFSET_PX: f32 = 2.0` (band `0.0..=4.0`); doc comment for `FOCUS_RING_COLOR` explicitly states friend-game scope and that `QA-COND-0005` is not advanced. Inline test `ac3_focus_ring_color_ratifies_spec_accent_palette_triple` + integration test `ac3_focus_ring_color_ratifies_spec_section_seven_accent_palette_triple` enforce the ratification. |
| AC4 — Named pressed tokens with defaults | PASS | `PRESSED_BG_TINT_ALPHA: f32 = 0.16` (band `0.08..=0.24`); `PRESSED_OFFSET_Y_PX: f32 = 1.0` (band `0.0..=2.0`); inline test `ac4_pressed_tokens_in_documented_range` + visual-state-ordering test `ac4_pressed_distinct_from_hover_for_visual_state_disambiguation` enforce. |
| AC5 — Named disabled tokens with defaults | PASS | `DISABLED_BG_TINT_ALPHA: f32 = 0.50`, `DISABLED_TEXT_ALPHA: f32 = 0.40`, `DISABLED_BORDER_ALPHA: f32 = 0.20`; inline test `ac5_disabled_tokens_in_documented_range` + `ac5_disabled_bg_is_heaviest_to_flatten_saturation` enforce. |
| AC6 — Export shape | PASS | `client/src/ui/design_tokens/mod.rs` declares `pub mod interaction_states;` alongside the existing Sprint 14 Tier 0 modules; integration test `ac1_ac6_module_exports_four_named_token_set_families` consumes every constant via the public path. |
| AC7 — Global UI spec amendment | PASS | New `## §11 Interaction State Primitives` section authored with Hover / Focus / Pressed / Disabled tables + canonical defaults + friend-game scope guard. §10 Primary / Secondary button affordance subsections flipped to "see §11" forward references. Spec Adoption Matrix row updated to cite §11. Integration test `ac7_spec_amendment_anchors_present_in_global_ui_design_spec` walks the spec file and asserts every required anchor. |
| AC8 — Integration test asserts primitive module shape | PASS | New bin `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs` registered under `[[test]] ui_clean_pass_interaction_state_primitives_test`; 8 tests pass (`cargo test -p client --test ui_clean_pass_interaction_state_primitives_test`); see `cargo-test-log.txt` below. |
| AC9 — No inline literal regressions on the module's own surface | PASS | Every numeric value in `interaction_states.rs` is published as a named `pub const NAME: TYPE = ...;`. Integration test `ac8_ac9_every_named_constant_carries_at_least_one_doc_comment_line` walks the module source and asserts each `pub const` has an immediately-preceding `///` doc comment. Companion grep test `ac9_module_publishes_every_required_token_family_prefix` asserts the canonical prefix set. |
| AC10 — Per-surface migration explicitly OUT OF SCOPE | PASS | `git diff origin/main -- client/src/ui/lobby.rs client/src/ui/shop_auction/mod.rs client/src/ui/hud/mod.rs client/src/presentation/` is empty (no output). |
| AC11 — Friend-game scope preserved | PASS | `git diff origin/main -- production/sprint-status.yaml` is empty. No `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` accept-risk disposition flipped. |
| AC12 — No release / playtest / final-art / two-client GAME_OVER claims | PASS | The worker run does not edit `production/sprint-status.yaml`, `production/sprints/*`, `production/qa/qa-plan-sprint-15.md`, `production/stage.txt`, or any `production/session-state/*` file. The closure paperwork (`/story-done`) is explicitly out of scope per the PROMPT 1005 forbidden list — no `/story-done`, smoke, team-QA, gate-check, or release-check is run by this worker. |

---

## cargo-check log (excerpt)

```text
$ CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  CARGO_INCREMENTAL=0 \
  RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE' \
  cargo check -p client
    Checking shared v0.1.0 (...\shared)
    Checking client v0.1.0 (...\client)
    Finished `dev` profile [optimized] target(s) in 12.26s
```

## cargo-fmt --check log

```text
$ CARGO_TARGET_DIR=... cargo fmt -p client -- --check
(no output — fmt clean)
```

## cargo-test log

```text
$ CARGO_TARGET_DIR=... cargo test -p client --test ui_clean_pass_interaction_state_primitives_test
   Compiling shared v0.1.0 (...\shared)
   Compiling client v0.1.0 (...\client)
    Finished `test` profile [optimized] target(s) in 1m 14s
     Running ..\tests\integration\ui_clean_pass\interaction_state_primitives_test.rs

running 8 tests
test ac1_ac6_module_exports_four_named_token_set_families ... ok
test ac2_ac4_ac5_audit_array_alphas_in_unit_interval ... ok
test ac3_ac4_audit_array_pixels_non_negative_and_bounded ... ok
test ac4_ac5_pressed_disabled_visual_state_ordering_holds ... ok
test ac3_focus_ring_color_ratifies_spec_section_seven_accent_palette_triple ... ok
test ac8_ac9_every_named_constant_carries_at_least_one_doc_comment_line ... ok
test ac9_module_publishes_every_required_token_family_prefix ... ok
test ac7_spec_amendment_anchors_present_in_global_ui_design_spec ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Inline module-test log

```text
$ CARGO_TARGET_DIR=... cargo test -p client --lib design_tokens::interaction_states
    Finished `test` profile [optimized] target(s) in 15.33s
     Running unittests src\lib.rs

running 10 tests
test ui::design_tokens::interaction_states::tests::ac2_ac4_ac5_every_audited_alpha_within_unit_interval ... ok
test ui::design_tokens::interaction_states::tests::ac2_hover_tokens_in_documented_alpha_range ... ok
test ui::design_tokens::interaction_states::tests::ac3_ac4_every_audited_pixel_non_negative_and_finite ... ok
test ui::design_tokens::interaction_states::tests::ac3_focus_ring_pixel_tokens_in_documented_range ... ok
test ui::design_tokens::interaction_states::tests::ac4_pressed_distinct_from_hover_for_visual_state_disambiguation ... ok
test ui::design_tokens::interaction_states::tests::ac4_pressed_tokens_in_documented_range ... ok
test ui::design_tokens::interaction_states::tests::ac3_focus_ring_color_ratifies_spec_accent_palette_triple ... ok
test ui::design_tokens::interaction_states::tests::ac5_disabled_bg_is_heaviest_to_flatten_saturation ... ok
test ui::design_tokens::interaction_states::tests::ac5_disabled_tokens_in_documented_range ... ok
test ui::design_tokens::interaction_states::tests::ac9_audit_arrays_match_published_token_counts ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 45 filtered out
```

---

## AC10 disjoint-surface git-diff verification

```text
$ git diff origin/main -- \
    client/src/ui/lobby.rs \
    client/src/ui/shop_auction/mod.rs \
    client/src/ui/hud/mod.rs \
    client/src/presentation/
(empty — no per-surface migration writes)
```

## AC11 accept-risk preservation

```text
$ git diff origin/main -- production/sprint-status.yaml
(empty — no row-status / accept-risk disposition flip)
```

## Worker no-claim preservation

This `/dev-story` worker run (PROMPT 1005) does **not** claim:

- Public release readiness or release-candidate readiness.
- Full game completion.
- Full playable-client manual QA.
- Standard-tier accessibility completion (`QA-COND-0005` preserved at
  the L5 `LOBBY_BUTTON_HEIGHT = 30.0` hit-target gap; visual focus ring
  presence does NOT advance Standard-tier focus-order conformance).
- Standard-tier hit-target conformance (≥44 px).
- Playtest / fun-hypothesis validation (`QA-COND-0006` preserved).
- Final-art / asset-production completion (`PAW-TD-*-a` preserved).
- Two-client GAME_OVER closure (`S8-QA-001-W1` OPEN).
- The `Polish->Release` gate-check retry (PROMPT 761 `FAIL` preserved).
- Stage advance from `Polish` to `Release`.
- Per-surface migration of any existing Sprint 14 button surface
  (deferred to Sprint 16+ family `S16-UI-INTERACTION-STATE-MIGRATION-*`).
- Closure of `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` itself — this
  is the `/dev-story` worker run only; `/story-done` is a separate
  paperwork prompt that this worker does NOT execute (per PROMPT 1005
  forbidden list).

---

## Stale target cleanup

Not required. D: free space at session start was 826 GB / 1.3 TB total,
well above the 40 GB threshold from the PROMPT 1005 disk rule. No
`Remove-Item` invocation.
