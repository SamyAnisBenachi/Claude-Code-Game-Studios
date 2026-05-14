# UI Clean-Pass Roadmap (PROMPT 802 Audit Reconciliation)

> **Story**: `S13-UI-AUDIT-ROADMAP-PREP-001` (`production/epics/ui-clean-pass/story-001-prompt-802-audit-roadmap-prep.md`)
> **Authoring prompt**: PROMPT 838 (this roadmap)
> **Source audit**: `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`
> **Source-of-truth at authoring**: `origin/main@4f7ba78` (`qa(s13): /story-done S11-SERVER-POOL-INIT-LOG-GUARD-001 (PROMPT 833)`)
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-ui-audit-roadmap-prep`
> **Branch**: `work/s13-ui-audit-roadmap-prep`
> **Stage**: `Polish` (unchanged; not advanced by this story)
> **Sprint**: Sprint 13 (Nice to Have row); the roadmap **does not activate** any of the 14 PROMPT 802 candidate slugs and **does not pull** any UI clean-pass work into Sprint 13 implementation

---

## Status / No-Claim Banner

This roadmap is **paperwork only**. It is the deliverable of the Sprint 13
`S13-UI-AUDIT-ROADMAP-PREP-001` Nice to Have story and is authored against
`origin/main@4f7ba78`.

**Sprint 13 does NOT attempt the full UI overhaul.** The 14 PROMPT 802
candidate slugs are not activated by this roadmap. Sprint 13 stage
remains `Polish` and is not advanced. Sprint 12 disposition
(`closed-with-conditions` per PROMPT 817) remains preserved unchanged.
Sprint 11 / Sprint 10 closeouts remain preserved unchanged. PROMPT 761
`Polish->Release` gate-check `FAIL` evidence preserved at
`production/gate-checks/gate-polish-release-2026-05-12.md`. No retry is
attempted by this roadmap.

This roadmap does **not** claim, advance, or close:

- Public release readiness.
- Release-candidate readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion (`PAW-TD-*-a`).
- PROMPT 685 8-story milestone closure (disposition documented only).
- Closure of any of the 14 PROMPT 802 candidate slugs.
- Sprint 13 stage advance or `Polish->Release` gate-check retry.

PROMPT 838 (this authoring run) does **not** invoke `/dev-story`,
`/story-readiness`, `/story-done`, `/smoke-check`, `/team-qa`,
`/gate-check`, `/release-check`, `/qa-plan`, or any cargo / trunk build
or test command. PROMPT 838 makes **no production-source change** under
`client/`, `server/`, `shared/`, or `tests/`. PROMPT 838 does **not**
modify `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
`production/sprints/sprint-12.md`, any other sprint plan,
`production/stage.txt`, `production/qa/qa-plan-sprint-13.md`,
PROMPT 761 gate-check artifact, `production/session-state/`, or
`.claude/settings.json`.

---

## Friend-Game Scope vs Standard-Tier-Accessibility Scope Boundary

This roadmap sequences UI-layout repair candidates for a **friend-game
product showcase** scope. Standard-tier accessibility work (WCAG
contrast ratios, ≥44px hit-targets, full keyboard navigation, screen
reader support, colorblind modes, text scaling) is **out of scope** and
remains `QA-COND-0005` accepted-risk.

Sprint 14+ activation prompts that pull rows from this roadmap **must
preserve this boundary**. Specifically:

- A future repair story that touches contrast, focus-order, or text-size
  in service of *visual hierarchy* does **not** thereby advance
  `QA-COND-0005`. The story must explicitly state the scope is
  friend-game visual polish only.
- The lobby `LOBBY_BUTTON_HEIGHT = 30.0` defect (PROMPT 802 §3.1 L5)
  remains **accepted-risk** under `QA-COND-0005` for friend-game scope.
  Pulling `S11-UX-LOBBY-CLASS-PICKER` or `S11-UX-LOBBY-BUTTON-HITTARGETS`
  into a sprint does **not** thereby commit to Standard-tier hit-target
  conformance.
- Final-art / asset-production (`PAW-TD-*-a` across PAW-002..PAW-006)
  remains **accept-risk**. PROMPT 802 §3.6 A1 (auction chrome reuses
  shop chrome under `PAW-TD-003-a`) and §3.5 S2 (shop slot wells use
  placeholder PAW-003 PNGs) are layout-differentiation problems with
  layout-only fixes; final-art replacement is a separate sprint scope.
- `QA-COND-0006` (playtest / fun-hypothesis validation) remains
  **accept-risk / deferred**; UI clean-pass repair does **not** advance
  playtest validation even when the surface is visibly polished.

If a Sprint 14+ activation attempts to silently expand the claim
(e.g. flips `QA-COND-0005` to `closed`, claims Standard-tier
conformance, or claims `PAW-TD-*-a` resolved by layout-only repair),
the activation must be rejected and the row sent back for scope
correction.

---

## Source Material

The roadmap reconciles three inputs:

| Input | Path | Role |
|---|---|---|
| PROMPT 802 audit | `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` | 14 candidate slugs sequenced in §3 + §4 + §6 |
| PROMPT 685 8-story milestone backlog | `production/sprints/sprint-11.md:279-286` and `production/sprints/sprint-12.md:385-392` (identical content) | Older UI clean-pass milestone (never authored into story files; never pulled into a sprint) |
| Route-readability future candidates | `production/qa/evidence/sprint-10-route-readability-notes.md` (referenced by PROMPT 802 §11) | 12 already-tracked Tier 2 eyeball / cosmetic capture rows |

### PROMPT 685 Canonical Source

PROMPT 823 / 824 hygiene confirmed **no `reports/PROMPT-685-*` file
exists on `origin/main`**. The canonical PROMPT 685 source is the
2026-05-11 "wave 10 / wave 11" 8-row UI clean-pass milestone backlog
recorded verbatim in the "Wider Sprint backlog (not yet pulled)" tail
section of:

- `production/sprints/sprint-11.md:279-286`
- `production/sprints/sprint-12.md:385-392`

Both locations carry identical content. The 8 rows (with constituent
slugs) are:

| PROMPT 685 row | Slugs |
|---|---|
| 1 | `S11-TD-UI-ZINDEX-LAYERS` |
| 2 | `S11-TD-UI-FLEX-STRIPS` + `S11-UX-HUD-TOP-STRIP-LAYOUT` + `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` + `S11-UX-HUD-OPP-FIGURINE` |
| 3 | `S11-UX-DRAFT-GRID-CENTERED-MODAL` |
| 4 | `S11-UX-AUCTION-FEATURED-CARD` + `S11-UX-AUCTION-FREE-GOLD-COUNTERS` |
| 5 | `S11-UX-LOBBY-CLASS-PICKER` + `S11-UX-LOBBY-BUTTON-HITTARGETS` |
| 6 | `S11-UX-BOARD-RENDERING-SPEC` |
| 7 | `S11-TD-UI-FONT-CONSTANTS` |
| 8 | `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` |

PROMPT 685 is treated as **`subsumed-by` PROMPT 802**: every row's
constituent slug(s) appear in the PROMPT 802 §4 ranked task list with a
re-validated verdict against `origin/main@b5eef0d`. No PROMPT 685 row
is `obsolete`; no row is `still valid (sequence after PROMPT 802)`.
PROMPT 685 disposition is **not closed** by this roadmap — only its
subsumption status is documented. See the
[PROMPT 685 -> PROMPT 802 Reconciliation Matrix](#prompt-685---prompt-802-reconciliation-matrix)
section below.

---

## The 14 PROMPT 802 Candidate Slugs (Sprint 14+ Pull-In Sequence)

This is the canonical sequenced set the roadmap commits to for Sprint
14+ activation. It is drawn from PROMPT 802 §6 (Lane A + Lane B + Lane
C; 12 story-authoring prompts) plus the two §4 Tier 3 refactor rows
that §6 sequences for activation after Tier 1 lands. Sequencing
dependencies from PROMPT 802 §5 / §8 are preserved (Tier 0 before
Tier 1; Tier 1 before Tier 3; Tier 0 design-token work mostly serial;
Tier 1 cross-surface parallel-safe after Tier 0 lands).

| Rank | Slug | Provenance | Tier | Priority | Effort | Subsumes PROMPT 685 row | Phase 1 dependency |
|---|---|---|---|---|---|---|---|
| 1 | `S11-TD-UI-ZINDEX-LAYERS` | PROMPT 685 row 1 (re-validated by PROMPT 802 §3.9 G1, §6 Lane A) | 0 | Must | 1.0d | row 1 | n/a (foundational) |
| 2 | `S11-TD-UI-FONT-CONSTANTS` | PROMPT 685 row 7 (re-validated by PROMPT 802 §3.9 G3, §6 Lane A) | 0 | Must | 0.5d | row 7 | n/a (foundational) |
| 3 | `S11-TD-UI-FLEX-STRIPS` | PROMPT 685 row 2 (re-validated by PROMPT 802 §3.9 G2, §6 Lane A) | 0 | Must | 1.0d | row 2 (partial) | n/a (foundational) |
| 4 | `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` | PROMPT 685 row 8 (re-validated by PROMPT 802 §3.9 G5, §6 Lane A) | 0 | Must | 1.0d | row 8 | n/a (foundational; parallel-safe with Tier 0.1-0.3) |
| 5 | `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` | Net-new (PROMPT 802 §3.2 H4, §3.9 G4, §6 Lane B) | 0 | Must | 0.25d | (none — net-new) | n/a (foundational) |
| 6 | `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` | Net-new (PROMPT 802 §3.9 G6, §6 Lane B, §9 producer-decision-2) | 0 | Must | 1.0d | (none — net-new) | n/a (foundational; should author first so Tier 0.2/0.3/0.5 have numeric inputs) |
| 7 | `S11-UX-HUD-TOP-STRIP-LAYOUT` | PROMPT 685 row 2 (re-validated by PROMPT 802 §3.2 H8, §6 Lane C) | 1 | Must | 0.75d | row 2 (HUD strip slice) | depends on ranks 1, 3, 6 |
| 8 | `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` | PROMPT 685 row 2 (re-validated by PROMPT 802 §3.2 H9, §6 Lane C) | 1 | Must | 0.5d | row 2 (HUD strip slice) | depends on ranks 1, 3, 6 |
| 9 | `S11-UX-DRAFT-GRID-CENTERED-MODAL` | PROMPT 685 row 3 (re-validated by PROMPT 802 §3.4 D1, §6 Lane C) | 1 | Must | 0.75d | row 3 | depends on ranks 1, 3, 4, 6 |
| 10 | `S11-UX-AUCTION-FEATURED-CARD` | PROMPT 685 row 4 (re-validated by PROMPT 802 §3.6 A2, §6 Lane C) | 1 | Must | 0.75d | row 4 (featured slice) | depends on ranks 1, 3, 6 |
| 11 | `S11-UX-LOBBY-CLASS-PICKER` | PROMPT 685 row 5 (re-validated by PROMPT 802 §3.1 L2, §3.1 L3, §6 Lane C) | 1 | Must | 1.0d | row 5 (class-picker slice) | depends on ranks 1, 3, 4, 6 |
| 12 | `S12-UX-LOBBY-LAYOUT-MODAL-001` | Net-new (PROMPT 802 §3.1 L1, §3.1 L4, §6 Lane C, §9 producer-decision-3) | 1 | Must | 1.0d | (none — net-new; producer must pick between modal-panel vs full-viewport hero layout per §9 producer-decision-3 before authoring) | depends on ranks 1, 3, 4, 6 |
| 13 | `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` | Net-new (PROMPT 802 §3.3 HA1, §3.3 HA5, §4 Tier 3.1) | 3 | Should | 1.5d | (none — net-new) | depends on ranks 1, 2, 3, 6 + at least one Tier 1 surface stable |
| 14 | `S11-UX-BOARD-RENDERING-SPEC` | PROMPT 685 row 6 (re-validated by PROMPT 802 §3.7 B1, §4 Tier 3.2) | 3 | Should | 0.75d | row 6 | depends on rank 6 (design-spec parent doc) |

**Total effort for the 14**: ~11.75d (subset of PROMPT 802 audit's
~17.5d grand total; excludes Tier 0 Should rank 0.6, Tier 1 Should
ranks 1.3 / 1.6 / 1.7 / 1.9 / 1.11, and Tier 2 cosmetic captures —
all of which are tracked separately below).

### Sequencing Rules (Preserved From PROMPT 802 §5 / §8)

1. **Tier 0 (ranks 1-6) lands first.** Phase 1 in PROMPT 802 §5
   nomenclature. Without these, every Tier 1 surface story has to
   either re-author primitives inline or skip the design-token
   integration — both reintroduce the original defects.
2. **Within Tier 0**, rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) is
   authored first because Tier 0 token modules need its numeric values
   as input (PROMPT 802 §9 producer-decision-2). Ranks 1-5 are partial-
   collision-prone (multiple stories touch a shared
   `client/src/ui/design_tokens/` host module) — per PROMPT 802 §8
   they run **mostly serial** within Tier 0. Rank 4 (viewport-
   invariant tests, new test bin) is parallel-safe with ranks 1-3.
3. **Tier 1 (ranks 7-12) waits for Tier 0.** Once Tier 0 lands, ranks
   7-12 each touch a different surface module (hud / shop_auction /
   lobby) so they are parallel-safe with each other within Tier 1.
4. **Tier 3 (ranks 13-14) lands last.** Rank 13
   (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`) refactor touches hand + shop +
   auction together (per PROMPT 802 §8), so it must wait for Tier 1
   surfaces to be stable. Rank 14 (`S11-UX-BOARD-RENDERING-SPEC`) is a
   doc-only spec authoring story that depends on rank 6.

### Tier 1 Should-Priority Adjacent Rows (PROMPT 802 §6 second-batch
candidates; not in the 14)

These five PROMPT 802 §3 / §4 Tier 1 candidate slugs are **not in the
14** because they are Should priority (not Must) and PROMPT 802 §6's
first author-batch (Lane C) only covers the six Tier 1 Must rows. They
are valid Sprint 14+ pull-in candidates but should not be activated
before the matching Tier 1 Must row on their surface lands.

| Slug | Tier | Priority | Effort | Pairs with | Source |
|---|---|---|---|---|---|
| `S11-UX-HUD-OPP-FIGURINE` | 1 | Should | 0.5d | Pair with ranks 7 / 8 | PROMPT 685 row 2 (opp-figurine slice) / PROMPT 802 §3.2 H10 |
| `S11-UX-AUCTION-FREE-GOLD-COUNTERS` | 1 | Should | 0.5d | Pair with rank 10 | PROMPT 685 row 4 (free-gold slice) / PROMPT 802 §3.6 A3 |
| `S11-UX-LOBBY-BUTTON-HITTARGETS` | 1 | Should | 0.25d | Pair with rank 11; note `QA-COND-0005` accept-risk preserved on the L5 hit-target defect | PROMPT 685 row 5 (button-hittargets slice) / PROMPT 802 §3.1 L5 |
| `S12-UX-AUCTION-LEAD-LOSS-STATE-001` | 1 | Should | 0.5d | Pair with rank 10; producer must pick visual language per §9 producer-decision-4 before authoring | Net-new / PROMPT 802 §3.6 A7 |
| `S12-UX-HAND-DRAG-STATE-VISUALS-001` | 1 | Should | 0.5d | Independent of the 14; orthogonal to ranks 7-12; touches hand UI only | Net-new / PROMPT 802 §3.3 HA3 |

### Tier 0 Should-Priority Adjacent Row (also not in the 14)

| Slug | Tier | Priority | Effort | Source |
|---|---|---|---|---|
| `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` | 0 | Should | 1.0d | Net-new / PROMPT 802 §3.9 G7. Pair with rank 6 (design spec); ranks 7-12 are tolerable without it but degrade to per-site button styling. |

### Tier 2 Cosmetic / Eyeball Captures (PROMPT 802 §4 Tier 2)

These 12 already-tracked future-candidate slugs are **out of the
roadmap's 14-slug Sprint 14+ MVP sequence** and are bundled per
PROMPT 802 §9 producer-decision-5 into a single proposed Sprint 14+
Should Have row, `S13-UX-CAPTURES-CLEAN-PASS-001` (per §9), or split
per-surface if the producer overrides. Most are 0.10d-0.25d manual
capture work. They can run before, during, or after Tier 1, including
as baseline captures of the current unrepaired state.

| Slug | Source |
|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | Already-tracked (resolved by Sprint 13 `S11-HUD-TIMER-EYEBALL-VISUAL-001` row at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` — outside this roadmap's scope) |
| `S11-UX-LOBBY-ROOM-CODE-EYEBALL-001` | Already-tracked future candidate |
| `S11-UX-LOBBY-OPP-SLOT-DISAMBIGUATION-001` | Already-tracked future candidate |
| `S11-DRAFT-INITIAL-OVERLAY-EYEBALL-001` | Already-tracked future candidate |
| `S11-UX-SHOP-SLOT-AFFORDANCE-001` | Already-tracked future candidate |
| `S11-UX-SHOP-INLINE-GOLD-READ-ORDER-001` | Already-tracked future candidate |
| `S11-UX-AUCTION-SETTLEMENT-VISUAL-EYEBALL-001` | Already-tracked; depends on `S8-QA-001-W1` (out of UI clean-pass scope) |
| `S11-HU-DRAG-FEEDBACK-DIFFERENTIATION-001` | Already-tracked future candidate (orthogonal to `S12-UX-HAND-DRAG-STATE-VISUALS-001`) |
| `S11-UX-RESULT-RETURN-TO-LOBBY-001` | Already-tracked future candidate; result screen is the only "acceptable" surface in PROMPT 802 §3.8 |
| `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` | Already-tracked future candidate |
| `S11-UX-BOARD-STATUS-ICON-LEGEND-001` | Already-tracked future candidate |
| `S11-UX-HUD-TIMER-URGENCY-VISUAL-001` | Already-tracked future candidate |

---

## 3-4 Highest-Impact Rows For Sprint 14 Must Have Framing

These rows are flagged as "must land before any polished
friend-game-product showcase". They are drawn from the 14-slug Sprint
14+ MVP sequence above. Sprint 14 activation should pull these as Must
Have rows; the remaining 10 rows from the 14 are Should/Nice
candidates within the same milestone.

1. **`S11-TD-UI-ZINDEX-LAYERS`** (rank 1, Tier 0, foundational).
   PROMPT 802 §3.9 G1: zero `ZIndex` / `GlobalZIndex` in
   `client/src/ui/`. Modal overlay, dim overlay, drag ghosts,
   settlement overlay, and toast all rely on spawn-order; reconnect /
   snapshot-rebuild can re-order them and visibly break the UI.
   PROMPT 802 §5 sequencing rule explicitly names z-index as the
   refactor that must land **first** because every Tier 1 surface
   story depends on it. Without this row, every other repair is at
   risk of regressing at the first reconnect.
2. **`S12-UX-LOBBY-LAYOUT-MODAL-001`** (rank 12, Tier 1, net-new).
   PROMPT 802 §3.1 L1: the lobby is a 420×?? top-left column on a
   1920×1080 viewport with the rest of the screen blank — the friend-
   game first-impression surface fails the polish bar before play
   begins. Sprint 14 cannot frame the build as a friend-game showcase
   while the lobby reads as an unfinished prototype panel.
   (Note: requires producer decision on layout direction per PROMPT
   802 §9 producer-decision-3 before authoring.)
3. **`S11-UX-AUCTION-FEATURED-CARD`** (rank 10, Tier 1, PROMPT 685).
   PROMPT 802 §3.6 A2: the auction is the highest-information-density
   UI moment in the game (30s `DraftAuction` timer + competing bids),
   and the featured auction-up card is visually indistinguishable from
   shop slot wells because both reuse the same placeholder chrome PNG.
   A friend-game showcase that records this moment puts the auction
   front-and-center; without featured-card differentiation the moment
   reads as flat / confused.
4. **`S11-UX-HUD-TOP-STRIP-LAYOUT`** (rank 7, Tier 1, PROMPT 685).
   PROMPT 802 §3.2 H1 / H8: the HUD is on-screen every frame of
   gameplay and its top strip (gold / mana / phase) is not composed
   via a flex parent — every line is its own absolute child with
   magic offsets. Every screenshot, every clip, and every showcase
   recording will display this. A friend-game polish claim that ships
   without this fix is visibly false on first frame.

Implicit rationale across all four: each row is **Must** priority in
PROMPT 802; each touches a different surface (foundational / lobby /
auction / HUD), so the four can be staged across the Sprint 14
critical path without single-file collision; and each is named in
PROMPT 802 §3 as a primary verdict driver for the "not acceptable as
polished product UI" verdict in §1.

---

## PROMPT 685 -> PROMPT 802 Reconciliation Matrix

The PROMPT 685 8-story milestone is **`subsumed-by` PROMPT 802 in
full**. Every row's constituent slug(s) appear in the PROMPT 802 §4
ranked task list and have been re-validated against the current code
on `origin/main@b5eef0d` (PROMPT 802 source-of-truth) and again
referenced against `origin/main@4f7ba78` (this roadmap source-of-
truth) — no slug has been retired or rendered obsolete by intervening
sprints. No row is `still valid (sequence after PROMPT 802)`; no row
is `obsolete`.

PROMPT 685 disposition is **not closed** by this roadmap. Only
subsumption status is documented.

| PROMPT 685 row | Slug(s) | Disposition | PROMPT 802 §4 rank(s) | This roadmap rank(s) |
|---|---|---|---|---|
| 1 | `S11-TD-UI-ZINDEX-LAYERS` | `subsumed-by S11-TD-UI-ZINDEX-LAYERS (re-validated by PROMPT 802 §3.9 G1, §4 Tier 0.1)` | 0.1 | 1 |
| 2 | `S11-TD-UI-FLEX-STRIPS` + `S11-UX-HUD-TOP-STRIP-LAYOUT` + `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` + `S11-UX-HUD-OPP-FIGURINE` | `subsumed-by S11-TD-UI-FLEX-STRIPS (Tier 0.3) + S11-UX-HUD-TOP-STRIP-LAYOUT (Tier 1.1) + S11-UX-HUD-BOTTOM-STRIP-LAYOUT (Tier 1.2) + S11-UX-HUD-OPP-FIGURINE (Tier 1.3) (re-validated by PROMPT 802 §3.2 H1, H8, H9, H10, §3.9 G2, §4)` | 0.3, 1.1, 1.2, 1.3 | 3, 7, 8, adjacent (Tier 1 Should not in the 14) |
| 3 | `S11-UX-DRAFT-GRID-CENTERED-MODAL` | `subsumed-by S11-UX-DRAFT-GRID-CENTERED-MODAL (re-validated by PROMPT 802 §3.4 D1, §4 Tier 1.4)` | 1.4 | 9 |
| 4 | `S11-UX-AUCTION-FEATURED-CARD` + `S11-UX-AUCTION-FREE-GOLD-COUNTERS` | `subsumed-by S11-UX-AUCTION-FEATURED-CARD (Tier 1.5) + S11-UX-AUCTION-FREE-GOLD-COUNTERS (Tier 1.6) (re-validated by PROMPT 802 §3.6 A2, A3, §4)` | 1.5, 1.6 | 10, adjacent (Tier 1 Should not in the 14) |
| 5 | `S11-UX-LOBBY-CLASS-PICKER` + `S11-UX-LOBBY-BUTTON-HITTARGETS` | `subsumed-by S11-UX-LOBBY-CLASS-PICKER (Tier 1.8) + S11-UX-LOBBY-BUTTON-HITTARGETS (Tier 1.9) (re-validated by PROMPT 802 §3.1 L2, L3, L5, §4); §3.1 L5 hit-target ≥44px scope remains QA-COND-0005 accept-risk per friend-game scope boundary above` | 1.8, 1.9 | 11, adjacent (Tier 1 Should not in the 14) |
| 6 | `S11-UX-BOARD-RENDERING-SPEC` | `subsumed-by S11-UX-BOARD-RENDERING-SPEC (re-validated by PROMPT 802 §3.7 B1, §4 Tier 3.2)` | 3.2 | 14 |
| 7 | `S11-TD-UI-FONT-CONSTANTS` | `subsumed-by S11-TD-UI-FONT-CONSTANTS (re-validated by PROMPT 802 §3.9 G3, §4 Tier 0.2)` | 0.2 | 2 |
| 8 | `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` | `subsumed-by S11-TD-UI-VIEWPORT-INVARIANT-TESTS (re-validated by PROMPT 802 §3.9 G5, §4 Tier 0.4)` | 0.4 | 4 |

**Coverage summary**: 12 distinct PROMPT 685 slugs across 8 rows.
PROMPT 802 §4 ranks each one; this roadmap places the 6 Must-priority
PROMPT 685 slugs (and 1 Should-priority PROMPT 685 slug `S11-UX-BOARD-
RENDERING-SPEC` rank 14) in the 14-slug Sprint 14+ MVP sequence; the
remaining 5 Should-priority PROMPT 685 slugs (`S11-UX-HUD-OPP-FIGURINE`,
`S11-UX-AUCTION-FREE-GOLD-COUNTERS`, `S11-UX-LOBBY-BUTTON-HITTARGETS`
— note the `QA-COND-0005` accept-risk on the hit-target slug — and
the Tier-0 sub-slices of row 2) are in the Tier 1 Should-priority
adjacent-rows table above.

---

## Accept-Risk Dispositions Preserved

This roadmap explicitly preserves the following accept-risk
dispositions across PROMPT 802 candidate authoring and Sprint 14+
activation:

- **`PAW-TD-002-a` … `PAW-TD-006-a`** placeholder-art accept-risk
  across PAW-002..PAW-006. UI clean-pass repair is layout / composition
  / hierarchy / typography / z-order work only and does **not** advance
  placeholder-art resolution. PROMPT 802 §7 places final-art work
  explicitly out of audit scope.
- **`QA-COND-0005`** Standard-tier-accessibility accepted-risk
  (friend-game scope only). Hit-target ≥44px (`LOBBY_BUTTON_HEIGHT`
  L5 defect), full keyboard navigation, screen reader support,
  colorblind modes, text scaling, and WCAG contrast remain out of
  scope. Pulling `S11-UX-LOBBY-BUTTON-HITTARGETS` from the adjacent-
  rows table does **not** by itself advance `QA-COND-0005`.
- **`QA-COND-0006`** playtest / fun-hypothesis validation accepted-
  risk. UI clean-pass polish is not a playtest gate. Even a fully
  polished UI does not by itself produce playtest evidence.

Sprint 14+ activation prompts that pull rows from this roadmap **must**
re-state each of these accept-risk dispositions on the activation
artifact, and must not flip any of them to `closed` without a separate
scoped sprint and gate-check evidence.

---

## Out Of Scope For This Roadmap

- Activation of any of the 14 PROMPT 802 candidate slugs (and the 6
  adjacent Should-priority slugs).
- Authoring of any new story file under
  `production/epics/hand-ui/`, `production/epics/hud/`,
  `production/epics/shop-auction-ui/`,
  `production/epics/board-rendering/`,
  `production/epics/playable-client/` (lobby surface), or
  `production/epics/ui-clean-pass/` other than this roadmap's
  paperwork.
- Any UI implementation, code edit, asset edit, or shader edit.
- Sprint 13 stage advance.
- `Polish->Release` gate-check retry.
- Closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, any
  `PAW-TD-*-a` row, TQ-S12-C1..C7, PROMPT 685, or the PROMPT 683-era
  runtime divergence question.
- Sprint 12 / Sprint 11 / Sprint 10 disposition change.
- `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` invocation
  on any candidate slug.

---

## Recommended Sprint 14 Activation Pattern

For the orchestrator that activates Sprint 14:

1. Pull the 4 highest-impact rows (above) as Sprint 14 Must Have rows.
2. Pull the remaining Tier 0 rows (ranks 2-6) as Sprint 14 Must Have
   foundational rows; Tier 0 must land before any Tier 1 row enters
   `/dev-story`.
3. Pull the remaining Tier 1 Must rows (ranks 8, 9) as Sprint 14
   Should Have, deferred to Sprint 15 if Tier 0 burn-down consumes
   the Sprint 14 capacity.
4. Defer Tier 3 rows (ranks 13, 14) to Sprint 15.
5. Bundle the 12 Tier 2 cosmetic captures into a single
   `S14-UX-CAPTURES-CLEAN-PASS-001` Should Have row, or split per-
   surface only if the producer overrides PROMPT 802 §9 producer-
   decision-5.
6. Resolve PROMPT 802 §9 producer-decisions 1-6 before
   `/dev-story` on any row that names them as a blocker (decisions 2
   for ranks 2 / 3 / 5; decision 3 for rank 12; decision 4 for the
   adjacent `S12-UX-AUCTION-LEAD-LOSS-STATE-001` row).
7. Author `/qa-plan sprint-14` after Sprint 14 story files exist and
   pass `/story-readiness`, and before any `/dev-story` on a UI clean-
   pass row.
8. Re-state every accept-risk disposition from this roadmap on the
   Sprint 14 activation artifact.

Sprint 13 is **not** to be re-activated or re-scoped by Sprint 14
pull-in. Sprint 13 close-out (when it happens) carries the
`S13-UI-AUDIT-ROADMAP-PREP-001` story as `done` with this roadmap as
evidence and does **not** carry the 14 PROMPT 802 candidate slugs.
