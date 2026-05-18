# Story 018: S17-UI-HUD-OPP-MANA-CLEANUP-001 -- HUD Opponent Figurine + OPP Label + Mana Duplicate Cleanup

> **Epic**: HUD
> **Story ID**: S17-UI-HUD-OPP-MANA-CLEANUP-001
> **Status**: Partial / In Progress -- Sprint 17 Should Have row activated by PROMPT 1099; PROMPT 1105 `/dev-story` worker delivered AC1 + AC2 + AC4..AC15 (HUD class-reveal projection + reconnect rebuild + FROZEN + ADR-001 invariant); PROMPT 1111 paperwork-only integration merged worker delta onto `origin/main@4bd4f56` via no-ff merge (PARTIAL disposition explicit in merge commit message); **AC3 (mana microbadge removal) explicitly NOT DELIVERED** -- escalated per the worker-contract pause-and-escalate branch because the spawn site lives in `client/src/ui/hand/mod.rs` reserve strip (out of scope for `client/src/ui/hud/`-owned worker). AC16 (HUD epic story count refresh) deferred to future `/story-done` paperwork. Row remains open / not closed; PROMPT 1112 records this partial disposition paperwork-only and does NOT run `/story-done`. Follow-up candidate slug: `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` (preferred per PROMPT 1111 recommendation) or `S17-UI-HAND-RESERVE-STRIP-CLEANUP-001` if Sprint 17 capacity allows pulling AC3 forward as a separate single-row story before close-out.
> **Layer**: HUD / Presentation (state reducer + UI reactor; no protocol change)
> **Type**: Tech Debt -- UI reactivity repair + duplicate-display cleanup
> **Sprint**: Sprint 17 Should Have row per `production/sprints/sprint-17.md` §"Should Have". Activation is a separate explicit prompt (PROMPT 1093 pattern).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> **Estimated effort**: ~0.5d (bundled AUDIT-1076-10 + -16 + -17; single owner client/src/ui/hud/)
> **Source audit**: PROMPT 1076 `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md` §"Per-finding evidence" AUDIT-1076-10 (P2), AUDIT-1076-16 (P3), AUDIT-1076-17 (P3)

---

## Status / No-Claim Banner

This story is a Sprint 17 Should Have **candidate** authored by
PROMPT 1095. **No sprint is activated by this authoring run.**
PROMPT 1095 does NOT modify `production/sprint-status.yaml`,
`production/sprints/sprint-17.md`, `production/sprints/sprint-16.md`,
`production/stage.txt`, any `production/session-state/*` file, any
QA-plan / smoke / Team-QA / gate-check / release-check artifact
under `production/qa/`, any code under `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`,
`.github/`, or `Trunk.toml`. PROMPT 1095 does NOT run
`/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `cargo`,
`trunk`, or any CI command.

This story does **not** claim: public release readiness, release-
candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), Standard-tier hit-target
conformance (>=44 px), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client
GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production
completion (`PAW-TD-*-a` — opponent figurine asset remains the
PAW-004 placeholder authored by Sprint 14 story 017 with no real-
art replacement), `Polish->Release` gate-check retry, stage advance
from Polish to Release, closure of the Sprint 12 story 019 underlying
drag-runtime bug, closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`,
closure of any of the 24 PROMPT 1022 audit findings, closure of any
SOURCE-1077-* finding, or closure of any AUDIT-1076-* finding outside
the three bundled here (-10 / -16 / -17).

**No optimistic client-side authority is introduced or proposed.**
No protocol shape change. No new server-authoritative state. No new
C2S / S2C message. Opponent class identity continues to arrive via
`S2CClassesRevealed` (and `S2CGameSnapshot` on reconnect) per
ADR-001 / ADR-002 / ADR-012.

Sprint 16 disposition `closed-with-conditions` per PROMPT 1082 +
PROMPT 1088 preserved unchanged. Sprint 15 / 14 / 13 / 12 / 11 / 10
dispositions preserved unchanged. PROMPT 761 Polish->Release gate-
check `FAIL` evidence preserved. `PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`, `TQ-S12-C1..C7` preserved verbatim.

---

## Source Findings

This story bundles three PROMPT 1076 findings under a single HUD
owner because all three are HUD UI reactivity / duplicate-display
defects with the same root cause class (HUD subscribers not re-
painting after a state event, OR a stale floating microbadge).

### AUDIT-1076-10 (P2) — Opponent figurine strip shows `?` after reveal

- **Audit location**:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-10.
- **Evidence**: every InSession snapshot top-left strip;
  `client-b.log:46` `S2CClassesRevealed map_len=2`.
- **Expected**: Once classes are revealed, the strip shows the
  opponent's class crest.
- **Actual**: Generic `?` overlays on 4-5 small grey circles in the
  top-left strip.
- **State / log correlation**: state correct (server broadcasts
  `S2CClassesRevealed map_len=2`), UI never repaints.
- **Likely owner**: `client/src/ui/hud/` (opponent figurine row).
- **Likely root cause**: subscriber to `S2CClassesRevealed` for
  hud-opponent does not trigger a re-skin. Per PROMPT 1076 §6
  minimal repair surface: `client/src/ui/hud/opponent_figurine*.rs`
  + `client/src/state/mod.rs` (`apply_classes_revealed` reducer).

### AUDIT-1076-16 (P3) — `OPP ?` in HUD post-reveal

- **Audit location**: same report §"Per-finding evidence"
  AUDIT-1076-16.
- **Severity**: P3 (cosmetic; same root cause as AUDIT-1076-10).
- **Minimal repair surface**: `client/src/ui/hud/opp_label*.rs`.
- **Behaviour**: HUD `OPP ?` label survives reveal — text is set to
  literal `"?"` at spawn and never re-skinned when classes lock.

### AUDIT-1076-17 (P3) — Duplicate / unanchored mana microbadge

- **Audit location**: same report §"Per-finding evidence"
  AUDIT-1076-17.
- **Severity**: P3.
- **Evidence**: snapshot 000020 — "Reserve 0 + / Current 2" floats
  above the modal between the HUD strip and the modal; HUD strip
  already shows "MANA 2 / 10".
- **Behaviour**: two duplicate displays of the same mana value, one
  of them visually unanchored.
- **Minimal repair surface**: `client/src/ui/hud/mana_*.rs`.

### Why bundled

PROMPT 1076 §7 ("Parallel Repair Plan") lists AUDIT-10 + AUDIT-16
as "single owner `client/src/ui/hud/`". AUDIT-17 is also HUD-owned
(mana microbadge). All three live under `client/src/ui/hud/` and
all three are reactivity / duplicate-display defects. Per Sprint
17 plan row `S17-UI-HUD-OPP-MANA-CLEANUP-001`: "HUD opponent
figurine + OPP label + mana duplicate cleanup -- opponent figurine
strip and OPP label must repaint after `S2CClassesRevealed`; the
floating 'Reserve 0 + / Current 2' mana microbadge must be removed
in favour of the canonical `MANA 2 / 10` HUD strip."

---

## Problem Class / Prevention Target

**Defect class**: HUD sub-systems do not subscribe to (or fail to
react to) a state event that should drive a re-skin / re-paint /
removal.

- AUDIT-1076-10 + -16: `S2CClassesRevealed` arrives and the
  reducer updates the local state, but two HUD subscribers
  (opponent figurine strip + OPP label) do not re-skin. Existing
  Sprint 14 story 017 `S11-UX-HUD-OPP-FIGURINE` already added an
  opponent figurine entity (PROMPT 968 / PROMPT 975 / PROMPT 976
  closure) but the repaint trigger on `S2CClassesRevealed` did
  not fire correctly in the run-7 user test; per the audit, the
  UI never repaints.
- AUDIT-1076-17: the floating mana microbadge is a stale duplicate
  of the canonical `MANA 2 / 10` HUD strip; it should be removed.

**Prevention target**:

1. **Opponent figurine + OPP label re-paint on
   `S2CClassesRevealed`** — wire the existing `S2CClassesRevealed`
   reducer (or add a new HUD subscriber to its sink) so the
   opponent figurine entity and the OPP label text both re-skin to
   the revealed opponent class.
2. **Mana microbadge removal** — delete the floating "Reserve 0 +
   / Current 2" microbadge entity (or merge its content into the
   canonical HUD MANA strip if any payload is currently unique to
   the microbadge, which the audit suggests is not the case).

---

## Context

### Existing surface

- **`client/src/ui/hud/`** — owns all HUD UI. Sub-modules at
  audit time (re-verify at activation HEAD):
  - `client/src/ui/hud/opponent_figurine*.rs` — opponent
    figurine entity introduced by Sprint 14 story 017
    (`S11-UX-HUD-OPP-FIGURINE`, closed PROMPT 976 on
    `origin/main@a3bc885`).
  - `client/src/ui/hud/opp_label*.rs` (or similar) — OPP text
    label.
  - `client/src/ui/hud/mana_*.rs` — current mana / reserve mana
    HUD display. Mana microbadge (floating "Reserve 0 + / Current
    2") lives somewhere here OR in a sibling overlay file; the
    implementing worker re-verifies.
- **`client/src/state/mod.rs`** — `apply_classes_revealed`
  reducer; receives `S2CClassesRevealed` and updates local state
  resources. The reducer may or may not currently fire a Bevy
  Event/Message that HUD systems consume; the implementing worker
  re-verifies.
- **`client/src/asset_wiring.rs`** — `hud_figurine_asset(class_id)`
  resolver (existing per Sprint 14 story 017 close-out).
- **`shared/src/protocol.rs`** — `S2CClassesRevealed` message
  definition. **No protocol shape change** in scope.
- **Sprint 14 story 017 `S11-UX-HUD-OPP-FIGURINE`** — already
  Done; opponent figurine entity exists. This row builds on that
  closure; it does NOT re-author or re-implement story 017.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hud.md` may need a one-line update
  documenting the OPP label / figurine re-paint contract. The
  implementing worker MAY append a one-line note; otherwise leave
  untouched.
- **ADR-021** (Presentation Layer Architecture): no schedule
  change. The new (or fixed) subscriber slots into the existing
  `PresentationSet::StateSync` set per Sprint 14 story 017
  precedent.
- **ADR-001** (Objective Identity Unicast): no change. Class
  identity is NOT objective identity; `was_fake` invariant
  preserved.
- **ADR-002** (Client-Server Authority): no change. Class identity
  is server-authoritative; the HUD reads `S2CClassesRevealed` /
  `S2CGameSnapshot` only.
- **ADR-012** (Lobby class-lock authority): no change. Class lock
  remains server-authoritative; this row only repaints the HUD on
  reveal.
- **TR-HUD-005** (real/fake identity never rendered on scoreboard)
  preserved unchanged.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on every `.rs` edit.
  `liv-bevy-lightyear` is NOT required for this row because no
  new Lightyear receiver is added — the existing
  `S2CClassesRevealed` drain in `client/src/state/mod.rs` is the
  source of truth.

### Control Manifest Rules

- Required: opponent figurine re-skins to the revealed opponent
  class when `S2CClassesRevealed` arrives. Re-skin uses the
  existing `hud_figurine_asset(class_id)` resolver.
- Required: OPP text label re-skins to the revealed opponent class
  display string (concrete display format TBD by implementing
  worker; consistent with the per-class display strings used by
  the Sprint 14 lobby / class picker).
- Required: the floating mana microbadge entity is removed. The
  canonical HUD `MANA 2 / 10` strip continues to render as the
  single mana display.
- Required: re-skin happens in `StateSync` (instantaneous, no
  tween — matches TR-HUD-008 + Sprint 14 story 017 AC4 / AC5
  precedent).
- Required: `S2CGameSnapshot` reconnect rebuild covers the OPP
  label + opponent figurine re-skin (ADR-011 + Sprint 14 story
  017 AC5 binding preserved).
- Required: FROZEN-on-GAME_OVER behaviour preserved
  (TR-HUD-009 / Sprint 14 story 017 AC6): after `phase ==
  GAME_OVER`, incremental class updates rejected; only
  `S2CGameSnapshot` can overwrite.
- Required: no new client-side opponent-class inference; class
  comes from `S2CClassesRevealed` / `S2CGameSnapshot` ONLY
  (ADR-002 + ADR-012 binding preserved; Sprint 14 story 017 AC8).
- Required: `PAW-TD-004-a` placeholder-art accept-risk preserved
  (figurine continues to use the PAW-004 placeholder asset; no
  real-art replacement).
- Required: `QA-COND-0005` preserved (no Standard-tier hit-target
  or contrast claim on the OPP label by this row).
- Required: `QA-COND-0006` preserved (no playtest validation
  claim).
- Forbidden: introducing a new C2S or S2C message for class
  reveal repaint. The existing `S2CClassesRevealed` drain
  suffices.
- Forbidden: changing `shared/src/protocol.rs` `S2CClassesRevealed`
  shape.
- Forbidden: modifying `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Forbidden: real-art production for the opponent figurine or
  OPP label (`PAW-TD-*-a` preserved).
- Forbidden: closure of any AUDIT-1076-* finding outside the three
  bundled (-10 / -16 / -17).
- Forbidden: closure of any SOURCE-1077-* finding.

---

## Story Classification

**Story type**: **Integration** (multi-file HUD repaint wiring +
microbadge removal; touches state reducer or its Bevy event sink
plus three HUD subscribers).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type" matrix, Integration stories require integration test OR
documented playtest (BLOCKING gate). This row delivers an
integration test bin.

This is **NOT** a:

- Logic-only story (multi-system wiring required).
- Visual / feel story (no shader, VFX, or animation curve change;
  re-skin is instantaneous per StateSync).
- Final-art story (`PAW-TD-*-a` preserved).
- Accessibility story (`QA-COND-0005` preserved).
- UI layout story (no menu / strip layout change; only re-skin
  reactivity).

---

## Dependencies and Parallelism

### Prerequisites

- **Sprint 14 story 017 `S11-UX-HUD-OPP-FIGURINE`** Done (closed
  PROMPT 976 on `origin/main@a3bc885f5f54e9b4e254d9abeb6b72a3e2321e8a`).
  This row builds on the existing opponent figurine entity; it
  does not re-author it.
- The `S2CClassesRevealed` message exists on `origin/main`
  (predates Sprint 17).

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must) | **YES** | disjoint (`asset_wiring.rs` + `shop_auction/mod.rs` + `hand/mod.rs`; not `client/src/ui/hud/`). |
| `S17-UI-CARD-SLOT-INSET-WIRING-001` (Should) | **YES** | disjoint. |
| `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (Should) | **PARTIAL** | both edit `client/src/ui/hud/` (marker split adds per-sub-surface markers under hud). Serialise. |
| `S17-UI-BID-BUTTON-PHASE-RACE-001` (Should) | **YES** | disjoint. |
| `S17-UI-MODAL-BLACK-SLAB-001` (conditional Must) | **YES** | disjoint. |
| `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (conditional Must) | **YES** | disjoint. |
| `S17-UI-HAND-B0004-CLEANUP-001` (Nice) | **YES** | disjoint (hand UI, not hud). |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` (Nice) | **YES** | disjoint. |
| `S17-SERVER-START-OF-TURN-DEBUG-001` (Nice) | **YES** | disjoint. |

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Opponent figurine re-skins on `S2CClassesRevealed`
  (AUDIT-1076-10)** -- DELIVERED by PROMPT 1105 worker (`c6b7d70`), integrated to `origin/main@4bd4f56` by PROMPT 1111. `sync_class_reveal_hud_system` at `client/src/ui/hud/mod.rs` applies `HudClassReveal.opponent` to the opponent figurine `ImageNode` via `hud_figurine_asset(opp_class)`. Integration test `ac1_opponent_figurine_reskins_on_classes_revealed` (`tests/integration/hud/opp_figurine_label_mana_repaint_test.rs`) PASS.: GIVEN the post-implementation client running
  through `Lobby -> Handshaking -> InSession`, WHEN
  `S2CClassesRevealed` arrives with `map_len=2`, THEN the existing
  opponent figurine entity (Sprint 14 story 017 `OpponentFigurineMarker`
  or equivalent) re-skins from the PAW-004 placeholder `?` asset
  to `hud_figurine_asset(opponent_class_id)`. Integration test
  asserts: a fixture `S2CClassesRevealed` event drained into the
  ECS world results in the opponent figurine entity's `ImageNode`
  set to the expected resolved asset path for the opponent class.

- [x] **AC2 -- OPP text label re-skins on `S2CClassesRevealed`
  (AUDIT-1076-16)** -- DELIVERED by PROMPT 1105 worker, integrated by PROMPT 1111. `sync_class_reveal_hud_system` applies `HudClassReveal.opponent` to the `opponent_gold_prefix` `Text` via `format_opp_class_display(opp_class)` returning `"OPP {ClassId:?}"`. Worker explicitly wrote class identity to the prefix entity (not the value entity) to preserve the existing opponent-gold readout contract guarded by `reconnect_snapshot_rebuild_test.rs`. Integration test `ac2_opp_text_label_reskins_on_classes_revealed` PASS.: GIVEN the same flow, WHEN
  `S2CClassesRevealed` arrives, THEN the HUD OPP text label
  (currently `"OPP ?"` per AUDIT-1076-16) re-skins to a display
  string carrying the opponent's class identity. Concrete display
  format chosen by implementing worker (e.g. `"OPP Iop"` /
  `"OPP Ecaflip"`), consistent with the per-class display strings
  used by the existing Sprint 14 lobby class picker. Integration
  test asserts: post-`S2CClassesRevealed` drain, the OPP label
  `Text` component contains the expected per-class display string.

- [~] **AC3 -- Mana microbadge removed (AUDIT-1076-17)** -- **NOT DELIVERED -- EXPLICITLY CARRIED**. PROMPT 1105 worker invoked the story's worker-contract pause-and-escalate branch upon locating the floating "Reserve N + / Current N" microbadge spawn site at `client/src/ui/hand/mod.rs` (`spawn_reserve_strip` around L3505, per-card `Reserve N Current N` text on L3530, updater around L4108-L4110), which is scope-forbidden for a `client/src/ui/hud/`-owned worker per the story's "Forbidden files" list. PROMPT 1111 integration explicitly did NOT touch `client/src/ui/hand/mod.rs` and recorded AC3 as carried forward in the no-ff merge commit message and report. Follow-up candidate slug: **`S18-UI-HAND-RESERVE-STRIP-CLEANUP-001`** (preferred per PROMPT 1111 recommendation) -- a hand-ui cleanup row owned by `client/src/ui/hand/` rather than `client/src/ui/hud/`. Producer may instead activate `S17-UI-HAND-RESERVE-STRIP-CLEANUP-001` if Sprint 17 has remaining capacity and the AC3 work is wanted before Sprint 17 close-out. AC3 closure remains gated on that follow-up row landing on `origin/main`; no `/story-done` for this row is authorised until AC3 lands or until the AC3 carry-forward is explicitly accepted-into-Sprint-18 by the producer.: GIVEN
  the post-implementation client running through `Placement` phase
  (the phase that produced snapshot 000020 in the audit), WHEN
  inspected via ECS query OR via a fresh QA snapshot, THEN there
  is no floating microbadge entity carrying the text "Reserve 0
  + / Current 2" (or equivalent). The canonical HUD `MANA 2 / 10`
  strip continues to render. If the microbadge currently carries
  any unique payload not represented in the canonical strip
  (worker re-verifies; the audit says it does not), the worker
  pauses and escalates; otherwise the microbadge is removed.

- [x] **AC4 -- Re-skin happens in StateSync (instantaneous)** -- DELIVERED. `sync_class_reveal_hud_system` is scheduled in `PresentationSet::StateSync` after `sync_gold_text_system` and `sync_figurine_image_system`; no `Animator` / tween. Integration test asserts the system completes the re-skin in a single frame.:
  GIVEN the new subscriber (or modified reducer), WHEN inspected,
  THEN the re-skin system runs in `PresentationSet::StateSync`
  (no `Animator`, no tween — matches TR-HUD-008 / Sprint 14
  story 017 AC4 precedent). Integration test asserts the system
  is scheduled in the expected set OR asserts the re-skin
  completes in a single frame.

- [x] **AC5 -- Reconnect rebuild covers OPP figurine + label
  (ADR-011 binding preserved)** -- DELIVERED. `sync_class_reveal_from_snapshot_system` (MessageDrain, after `handle_game_snapshot_system`) reads `MessageReader<PresentationGameSnapshotMessage>` and writes `HudClassReveal`. Always runs so `S2CGameSnapshot` reconnect rebuilds remain authoritative even at GAME_OVER. Integration test covers fixture `S2CGameSnapshot` drain populating both the figurine and the OPP label.: GIVEN an `S2CGameSnapshot` arrives
  mid-session containing the opponent's `ClassId`, WHEN the HUD
  rebuilds, THEN both the opponent figurine ImageNode AND the OPP
  text label are part of the rebuild with the snapshot-correct
  class. Integration test asserts: a fixture `S2CGameSnapshot`
  drain populates both the figurine and the OPP label.

- [x] **AC6 -- FROZEN-on-GAME_OVER preserved
  (TR-HUD-009 / Sprint 14 story 017 AC6 binding)** -- DELIVERED. `sync_class_reveal_from_lobby_view_system` skips while `HudMode::Frozen` so incremental lobby reveals cannot overwrite during GAME_OVER. `sync_class_reveal_from_snapshot_system` always runs so an `S2CGameSnapshot` reconnect can still overwrite. Integration test covers both branches.: GIVEN `phase
  == GAME_OVER`, WHEN a hypothetical incremental
  `S2CClassesRevealed` arrives (e.g. via a malformed test
  fixture), THEN neither the opponent figurine nor the OPP label
  is updated. Only `S2CGameSnapshot` can overwrite. The
  implementing worker preserves the existing FROZEN check on the
  Sprint 14 story 017 figurine path and extends it to cover the
  OPP label re-skin.

- [x] **AC7 -- No client-side opponent-class inference added
  (ADR-002 + ADR-012 + Sprint 14 story 017 AC8 binding)** -- DELIVERED. The re-skin path reads from `Res<LobbyViewState>.revealed_classes` + `Res<ClientSessionIdentity>` (both populated by the existing `apply_classes_revealed` reducer that drains `S2CClassesRevealed`) and from the `S2CGameSnapshot` drain. No spawned-unit / lane-state / observation-derived inference introduced.: GIVEN
  `git diff <activation HEAD>..HEAD` for the worker's commit,
  WHEN inspected, THEN no system derives opponent class from
  spawned units, lane state, or any other client-side
  observation. The re-skin path reads from
  `S2CClassesRevealed` / `S2CGameSnapshot`-derived local state
  ONLY.

- [x] **AC8 -- ADR-001 invariant preserved** -- DELIVERED. `HudClassReveal { local: Option<ClassId>, opponent: Option<ClassId> }` carries class identity only; no objective identity / `was_fake` data flows to the OPP label or opponent figurine. Defence-in-depth grep recorded in the PROMPT 1105 evidence document.: GIVEN the post-
  refactor build, WHEN any path that surfaces the OPP label or
  opponent figurine is inspected, THEN no objective identity or
  `was_fake` data flows to either. The OPP label / figurine carry
  class identity only, NOT objective identity. Defence-in-depth
  grep + code review recorded in the evidence document.

- [x] **AC9 -- Integration test bin authored** -- DELIVERED. `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs` (NEW, 8 tests covering AC1 / AC2 / AC4 / AC5 / AC6 / AC7 / AC8 + opponent figurine marker singleton guard). PROMPT 1111 integration `cargo test -p client --test hud_opp_figurine_label_mana_repaint_test` PASS 8/8 plus 27/27 sub-tests across 6 sibling HUD test bins. AC3 microbadge-removal coverage explicitly NOT included in the new test bin and carries forward with AC3.: GIVEN
  `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs`
  (NEW; or split into per-defect bins under
  `tests/integration/hud/`), WHEN run, THEN it asserts AC1, AC2,
  AC3, AC4, AC5, AC6, AC7, AC8 against a real Bevy 0.18 `App`
  per the `tests/integration/hud/` pattern.

- [x] **AC10 -- No protocol or server change** -- DELIVERED. PROMPT 1111 integration `git diff --name-only origin/main...HEAD` returns exactly 4 paths: `client/Cargo.toml` + `client/src/ui/hud/mod.rs` + `production/qa/evidence/sprint-17-hud-opp-mana-cleanup/evidence.md` + `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs`. Zero changes under `server/`, `shared/`, or `tests/integration/server/`.: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN there
  are zero changes under `server/`, `shared/`, or
  `tests/integration/server/`. The implementation is client-side
  only.

- [x] **AC11 -- ADR-021 schedule preserved** -- DELIVERED. New systems slot into existing `PresentationSet::MessageDrain` (`sync_class_reveal_from_lobby_view_system`, `sync_class_reveal_from_snapshot_system`) and `PresentationSet::StateSync` (`sync_class_reveal_hud_system`) per ADR-021 + Sprint 14 story 017 precedent. No new schedule wiring introduced; PROMPT 1111 integration `cargo check -p client` PASS in 8.78s with zero warnings.: GIVEN `cargo build
  -p client` under the Cargo resource policy, WHEN run, THEN no
  new system-set or schedule wiring is introduced. The new
  subscriber slots into `PresentationSet::StateSync` (or the
  existing reducer system) per ADR-021.

- [x] **AC12 -- No accept-risk closure claimed** -- DELIVERED. PROMPT 1105 worker commit message + PROMPT 1111 integration merge commit + PROMPT 1112 partial-disposition paperwork all explicitly preserve `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a` (specifically `PAW-TD-004-a` opponent figurine placeholder), `TQ-S12-C1..C7`, PROMPT 761 `Polish->Release` FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all AUDIT-1076-* findings outside the three bundled (-10 / -16 / -17 with AC3 explicitly carried), all SOURCE-1077-*, all 24 PROMPT 1022 findings. Final-art replacement of the opponent figurine remains out of scope; Standard-tier hit-target conformance on the OPP label NOT pursued; playtest validation NOT pursued.: GIVEN the
  commit message and any evidence document, WHEN inspected, THEN
  they explicitly do NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a` (specifically
  `PAW-TD-004-a` for the figurine placeholder), or any other
  accept-risk disposition. Final-art replacement of the opponent
  figurine is explicitly out of scope. Standard-tier hit-target
  conformance on the OPP label is NOT pursued. Playtest
  validation is NOT pursued.

- [x] **AC13 -- Sprint 17 disposition preserved** -- DELIVERED at worker + integration time. PROMPT 1105 worker + PROMPT 1111 integration diffs touched zero files under `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md`. PROMPT 1112 is the first authorised modifier of `production/sprint-status.yaml` + `production/session-state/*` for this row, and applies paperwork-only partial-disposition edits (no `/story-done` close).: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/*`, `production/gate-checks/*`, and
  `docs/architecture/adr-*.md` are diffed, THEN none are modified
  by this story's `/dev-story` worker.

- [x] **AC14 -- Worker branch scope contained** -- DELIVERED. PROMPT 1105 worker pushed `work/s17-hud-opp-mana-cleanup` (`c6b7d70`) only -- never `main`. Files changed at worker time: `client/src/ui/hud/mod.rs`, `client/Cargo.toml` (dev-dependency wiring for new test bin), `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs` (NEW), `production/qa/evidence/sprint-17-hud-opp-mana-cleanup/evidence.md` (NEW). `client/src/state/mod.rs` `apply_classes_revealed` reducer NOT modified (worker chose to read via `Res<LobbyViewState>` instead of extending the reducer).: GIVEN the worker
  branch (slug recommendation:
  `work/s17-hud-opp-mana-cleanup`), WHEN inspected, THEN it
  pushes only the worker branch — never `main`. Files changed at
  worker time are scoped to `client/src/ui/hud/opponent_figurine*.rs`,
  `client/src/ui/hud/opp_label*.rs`, `client/src/ui/hud/mana_*.rs`,
  optionally `client/src/state/mod.rs` (the
  `apply_classes_revealed` reducer), and the new test bin under
  `tests/integration/hud/`.

- [x] **AC15 -- Cargo resource policy applied for every Cargo
  command** -- DELIVERED at worker + integration. PROMPT 1105 worker + PROMPT 1111 integration both applied the 5 Cargo resource policy env vars (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before every cargo invocation. PROMPT 1111 integration cargo gates: `cargo check -p client` OK 8.78s 0 errors 0 warnings; 7 targeted test bins PASS 35/35 sub-tests; D: free ~745 GB at start. PROMPT 1112 itself does NOT invoke cargo (paperwork-only).: future implementation MUST set the Cargo resource
  policy env vars (`CARGO_TARGET_DIR=
  D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before
  every `cargo check` / `cargo test` invocation on Windows /
  MSVC. Story authoring (PROMPT 1095) does NOT invoke Cargo.

- [ ] **AC16 -- HUD epic story count refreshed** -- DEFERRED to future `/story-done` paperwork (gated on AC3 closure or AC3-carry-accept disposition). PROMPT 1112 partial-disposition paperwork does NOT modify `production/epics/hud/EPIC.md`; this row's HUD epic story count refresh happens at proper `/story-done` time once AC3 lands or is explicitly accepted-into-Sprint-18.: GIVEN the
  `production/epics/hud/EPIC.md` Stories table, WHEN inspected
  after `/story-done` paperwork closure (post-`/dev-story`), THEN
  it reflects this new story (`018: ...`) and any consequent
  refresh of the `HUD_ENTITY_COUNT` summary line if the mana
  microbadge removal changes the pre-pooled count. The story file
  itself does NOT modify EPIC.md at authoring time; the
  `/story-done` paperwork prompt does the refresh.

---

## Implementation Notes

### Owned files (likely change set)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/hud/opponent_figurine*.rs` | Add or fix the `S2CClassesRevealed` subscriber that re-skins the existing opponent figurine entity to `hud_figurine_asset(opponent_class_id)`. Preserve FROZEN-on-GAME_OVER check (Sprint 14 story 017 AC6). |
| `client/src/ui/hud/opp_label*.rs` (or wherever the OPP label entity lives — re-verify at activation HEAD) | Add or fix the `S2CClassesRevealed` subscriber for the OPP text label re-skin. Same FROZEN check. |
| `client/src/ui/hud/mana_*.rs` (or wherever the floating mana microbadge is spawned) | Remove the floating microbadge entity. If the microbadge's spawn site is in a sibling file, the worker removes the spawn site there. |
| `client/src/state/mod.rs` (`apply_classes_revealed` reducer) | Possibly extend to write a Bevy Event/Message that HUD subscribers consume. Implementation choice TBD by worker (extend reducer vs add new HUD-side subscriber on `MessageReader<S2CClassesRevealed>`). |
| `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs` (NEW) | AC1-AC8 integration coverage. |
| `production/qa/evidence/sprint-17-hud-opp-mana-cleanup/evidence.md` (NEW, by `/dev-story` worker) | Evidence document; NOT authored by PROMPT 1095. |
| `production/epics/hud/EPIC.md` Stories table | Story row addition at `/story-done` time only — NOT at `/dev-story` time. |

### Forbidden files

- Everything under `server/`, `shared/`.
- Everything under `tests/integration/server/`,
  `tests/unit/server/`, `tests/integration/lightyear*`,
  `tests/unit/lightyear*`.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files (no cross-epic edit).
- `assets/art/lobby/*.png` and `assets/art/cards/display/*.png`
  (no real-art production; `PAW-TD-*-a` preserved).
- `docs/architecture/adr-*.md` (no ADR amendment in scope).
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.

### Cargo resource policy

Per the binding Sprint 15+ QA plan precedent, every `cargo`
invocation on Windows / MSVC MUST set the five env vars under
AC15.

### Target citations

- Sprint 17 plan row source:
  `production/sprints/sprint-17.md` §"Should Have" row
  `S17-UI-HUD-OPP-MANA-CLEANUP-001`.
- Source audit:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-10 / 16 / 17.
- Predecessor: Sprint 14 story 017
  `S11-UX-HUD-OPP-FIGURINE` (PROMPT 976 closure on
  `origin/main@a3bc885`).

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a
   fresh worktree (suggested slug
   `work/s17-hud-opp-mana-cleanup`).
2. Read this story file end-to-end before any code change.
3. Read Sprint 14 story 017 (`production/epics/hud/story-017-hud-opponent-figurine.md`)
   end-to-end to inherit the existing FROZEN-on-GAME_OVER + ADR-001
   binding pattern.
4. Re-verify the audit-time module shapes by reading the current
   `client/src/ui/hud/`, `client/src/state/mod.rs`, and the
   `S2CClassesRevealed` drain path. Sprint 17 conditional Must
   Have rows may have shifted file shapes.
5. Locate the floating mana microbadge spawn site. If not under
   `client/src/ui/hud/`, pause and escalate (the audit suggests
   it is HUD-owned; if it is elsewhere — e.g. in an overlay file
   — file ownership for AC3 needs clarification).
6. Activate `liv-bevy-018` skill before any `.rs` edit.
7. Pick the subscriber pattern (extend
   `apply_classes_revealed` reducer to write a Bevy Event AND
   have HUD systems read it, OR add a HUD-side
   `MessageReader<S2CClassesRevealed>` directly). Justify in the
   commit message.
8. Set the Cargo resource policy env vars per AC15 before every
   `cargo check` / `cargo test` invocation.
9. Run `cargo check -p client` and the targeted `cargo test -p
   client --test opp_figurine_label_mana_repaint_test` (or
   equivalent) under the Cargo resource policy.
10. Push the worker branch (never `main`).
11. Stop. Closure paperwork (`/story-done`, integration `/no-ff`
    merge) is a later prompt's scope.

The worker MUST NOT:

- Modify `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Modify `shared/src/protocol.rs` `S2CClassesRevealed` shape.
- Introduce a new C2S or S2C message.
- Touch `assets/art/lobby/*.png` or any other production art
  asset.
- Modify Cargo / Trunk / CI files.
- Modify `production/sprint-status.yaml`, `production/sprints/`,
  `production/stage.txt`, `production/session-state/`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation
  (targeted bins only).
- Run `trunk` or any CI command.
- Push to `main`.
- Claim closure of Sprint 14 story 017 (already Done; preserved
  unchanged).
- Claim closure of any AUDIT-1076-* finding outside -10 / -16 /
  -17.
- Claim release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, final-art completion,
  or stage advance.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to files this
worker owns (under `client/src/ui/hud/` + optionally
`client/src/state/mod.rs`) plus the new test bin. The worker
MUST NOT block on workspace-wide compilation errors introduced by
other in-flight Sprint 17 workers' branches. Per the Parallelism
summary above, this row is file-disjoint with every Sprint 17
row except `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (which also
edits `client/src/ui/hud/`); the orchestrator schedules those
two serially.

### Relay / reporting expectation for future workers

Final status line:

```
N: S17-UI-HUD-OPP-MANA-CLEANUP-001: STATUS
```

where `N` is the prompt number that ran `/dev-story`.

---

## Partial Integration Notes (PROMPT 1112)

Updated 2026-05-18 by PROMPT 1112 paperwork-only partial disposition.
Source-of-truth at this disposition: `origin/main@4bd4f569bf0f8e54a18b6f1a9c95336aefff34d9`
(PROMPT 1111 integration tip `integrate(s17): merge PROMPT 1105 HUD
class-reveal projection (PARTIAL, AC3 carried) into main (PROMPT 1111)`
merging PROMPT 1105 worker `c6b7d70a2733c1fa3b0af271c8e309397cf592a6`
`dev-story(s17-hud-opp-mana-cleanup): HUD class-reveal projection for
opp figurine + OPP label (PROMPT 1105)` onto `origin/main` via no-ff
merge; strict fast-forward descendant of `origin/main@30f166f` PROMPT
1106 card-slot inset wiring integration tip, of `origin/main@9a9b1dc`
PROMPT 1110 card-slot inset wiring story-done tip, of `origin/main@72d56bc`
PROMPT 1108 server start-of-turn-debug story-done tip, and of
`origin/main@dc8adb6` PROMPT 1107 server warn->debug integration tip).

### Disposition

**Partial / In Progress -- AC3 explicitly carried; row remains open;
NOT closed by `/story-done`.**

### AC status table

| AC | Delivered? | Worker / integration evidence |
|---|---|---|
| AC1 opponent figurine re-skin | DELIVERED | PROMPT 1105 worker `c6b7d70` + PROMPT 1111 integration `4bd4f56`; test `ac1_opponent_figurine_reskins_on_classes_revealed` PASS |
| AC2 OPP label re-skin | DELIVERED | Same; test `ac2_opp_text_label_reskins_on_classes_revealed` PASS |
| AC3 mana microbadge removal | **NOT DELIVERED -- EXPLICITLY CARRIED** | Spawn site at `client/src/ui/hand/mod.rs` reserve strip (L3505 / L3530 / L4108-L4110) is out of scope for `client/src/ui/hud/`-owned worker per the story's "Forbidden files" list; worker invoked the pause-and-escalate branch; PROMPT 1111 integration explicitly did NOT touch `client/src/ui/hand/mod.rs` and recorded AC3 as carried in the merge commit message |
| AC4 re-skin in StateSync | DELIVERED | `sync_class_reveal_hud_system` scheduled in `PresentationSet::StateSync` |
| AC5 reconnect rebuild covers OPP figurine + label | DELIVERED | `sync_class_reveal_from_snapshot_system` (MessageDrain after `handle_game_snapshot_system`); integration test PASS |
| AC6 FROZEN-on-GAME_OVER preserved | DELIVERED | Lobby reveal path skips while `HudMode::Frozen`; snapshot reconnect path always runs |
| AC7 no client-side opponent-class inference | DELIVERED | Reads from `Res<LobbyViewState>.revealed_classes` + `Res<ClientSessionIdentity>` only |
| AC8 ADR-001 invariant preserved | DELIVERED | `HudClassReveal` carries only `ClassId`, never objective identity / `was_fake` |
| AC9 integration test bin authored | DELIVERED | `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs` (NEW, 8 tests covering AC1 / AC2 / AC4 / AC5 / AC6 / AC7 / AC8 + marker singleton guard) |
| AC10 no protocol or server change | DELIVERED | Integration diff: 4 paths total, zero under `server/` / `shared/` / `tests/integration/server/` |
| AC11 ADR-021 schedule preserved | DELIVERED | New systems slot into existing `PresentationSet::MessageDrain` + `PresentationSet::StateSync` |
| AC12 no accept-risk closure claimed | DELIVERED | Worker + integration + this partial disposition preserve `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 761 FAIL, S11-HUD-TIMER carry, all AUDIT-1076-*, all SOURCE-1077-*, 24 PROMPT 1022 findings |
| AC13 Sprint 17 disposition preserved by worker + integration | DELIVERED | Worker + integration touched zero files under `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md`; PROMPT 1112 is the first authorised modifier of sprint-status + session-state for this row and applies paperwork-only partial edits |
| AC14 worker branch scope contained | DELIVERED | PROMPT 1105 worker pushed `work/s17-hud-opp-mana-cleanup` (`c6b7d70`) only -- never `main`; 4 files in scope; `client/src/state/mod.rs` reducer NOT modified |
| AC15 Cargo resource policy applied | DELIVERED | Worker + integration both applied all 5 env vars; integration `cargo check -p client` OK 8.78s 0/0; 7 targeted test bins PASS 35/35 sub-tests |
| AC16 HUD epic story count refresh | DEFERRED | Gated on `/story-done` paperwork once AC3 lands or AC3 carry-forward is explicitly accepted-into-Sprint-18 by producer |

### AC3 carry-forward classification and follow-up candidate

- **Location of remaining work**: `client/src/ui/hand/mod.rs` reserve strip
  (`spawn_reserve_strip` around L3505, per-card `Reserve N Current N`
  text on L3530, updater around L4108-L4110). Semantically distinct
  from HUD canonical current/cap mana strip (`MANA n / N`) which is
  preserved unchanged.
- **Suggested follow-up story slug**: **`S18-UI-HAND-RESERVE-STRIP-CLEANUP-001`**
  (preferred per PROMPT 1111 recommendation -- Sprint 17 is in Polish
  and AC3 is a hand-ui cleanup decision rather than a hot defect).
  Alternative slug `S17-UI-HAND-RESERVE-STRIP-CLEANUP-001` only if a
  producer explicitly authorises pulling AC3 forward as a separate
  single-row Sprint 17 story before close-out and Sprint 17 has
  remaining capacity. Either slug is a candidate only; PROMPT 1112
  does NOT author or activate the row.
- **/story-done deferral**: Closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001`
  is gated on either (a) AC3 landing on `origin/main` via a future
  hand-ui worker + integration, OR (b) the producer explicitly
  accepting the AC3 carry-forward into Sprint 18 (in which case
  `/story-done` may close the row as Done-with-AC3-carried-to-S18
  with the explicit producer decision recorded). PROMPT 1112 chooses
  neither path and records partial disposition only.

### Files changed by PROMPT 1112

- `production/epics/hud/story-018-opp-figurine-mana-cleanup.md` (Status banner updated; AC1 + AC2 + AC4..AC15 flipped to `[x]` with delivery evidence; AC3 flipped to `[~]` with explicit carry note; AC16 left `[ ]` with deferred-to-/story-done rationale; this Partial Integration Notes section appended; final status line flipped DRAFT -> PARTIAL).
- `production/sprint-status.yaml` (S17-UI-HUD-OPP-MANA-CLEANUP-001 row in stories: block flipped status: `ready` -> `in_progress` with `worker_prompt: "1105"`, `worker_branch`, `worker_commit`, `worker_report`, `integration_prompt: "1111"`, `integration_branch`, `integration_commit`, `integrated_commit: 4bd4f56`, `partial_disposition` note explaining AC3 carry; `sprint_17_partial_disposition:` block appended at EOF following the existing `sprint_17_story_done:` precedent pattern with full AC table, AC3 carry classification, follow-up candidate slugs, conditions carried forward, explicitly not claimed, files changed, forbidden changes observed sections).
- `production/session-state/active.md` (PROMPT 1112 banner prepended above PROMPT 1110 banner).
- `production/session-state/codex-orchestrator-state.md` (PROMPT 1112 paragraph prepended above PROMPT 1110 paragraph).
- `reports/PROMPT-1112-s17-hud-opp-mana-partial-disposition.md` (mandatory final report; gitignored).

### Forbidden changes observed by PROMPT 1112

- `client/**`, `server/**`, `shared/**`, `tests/**` NOT modified (paperwork-only; zero code changes).
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `Trunk.toml`, `.github/` NOT modified.
- `production/stage.txt` NOT modified (remains `Polish`).
- `production/sprints/sprint-17.md` NOT modified.
- `production/qa/qa-plan-sprint-17.md` NOT modified.
- `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` NOT modified.
- `production/gate-checks/*` NOT modified (PROMPT 761 `Polish->Release` FAIL preserved; NO retry).
- `docs/architecture/adr-*.md` NOT modified.
- No Sprint 17 story file under `production/epics/` other than this one modified.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` story 014 NOT modified (human-operator-blocked carry preserved verbatim).
- Existing `sprint_17_activation:` / `sprint_17_story_done:` PROMPT 1108 + PROMPT 1110 entries / `sprint_16_*` / `sprint_15_*` / `sprint_14_*` / `sprint_13_*` blocks in `production/sprint-status.yaml` NOT modified (preserved verbatim above the new `sprint_17_partial_disposition:` block).
- No `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` run by PROMPT 1112.
- No cargo / trunk / CI command run by PROMPT 1112.
- No Polish->Release retry.
- `S17-UI-HUD-OPP-MANA-CLEANUP-001` explicitly NOT closed as Done.

---

## Closure Trail

Closure trail is appended by future `/story-readiness`,
`/dev-story`, `/story-done`, and partial-disposition prompts.
PROMPT 1112 (this revision) records the partial disposition above;
final `/story-done` closure is deferred per the AC3 carry-forward
gating described in §"AC3 carry-forward classification and follow-up
candidate".

### Conditions carried forward unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved (specifically
  `PAW-TD-004-a` for the figurine placeholder).
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 14 story 017 `S11-UX-HUD-OPP-FIGURINE` Done (closed
  PROMPT 976; UNCHANGED).
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-
  blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT
  closed by this row.

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Sprint 17 close-out.
- Sprint 14 story 017 re-author / re-implementation.
- Closure of any AUDIT-1076-* finding outside -10 / -16 / -17.
- Closure of any SOURCE-1077-* finding.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.

`018: S17-UI-HUD-OPP-MANA-CLEANUP-001: PARTIAL`
