# Friend-Game Route Readability Notes (S11-ROUTE-READABILITY-CARRY-001)

> **Created**: 2026-05-13 (PROMPT 772 — Sprint 11 draft Must Have carry
> `S11-ROUTE-READABILITY-CARRY-001`, deferred from Sprint 10 nice-to-have
> `S10-N2` per PROMPT 763 close-out and folded into the Sprint 11 draft plan
> by PROMPT 764).
> **Source-of-truth at authoring**: `origin/main` HEAD at push time
> (post-PROMPT 770 `0d19690` + PROMPT 771 evidence-index commit).
> **Sprint 10 disposition**: `closed-with-conditions` per PROMPT 763.
> **Stage**: `Polish`. `production/stage.txt` unchanged.
> **Sprint 11 disposition**: `draft / not_active` (PROMPT 764). These notes do
> **not** activate Sprint 11, do **not** mutate
> `production/sprint-status.yaml`, do **not** mutate
> `production/sprints/sprint-11.md`, do **not** mutate
> `production/stage.txt`, and do **not** run `/dev-story`, `/story-done`,
> `/smoke-check`, `/team-qa`, `/gate-check`, or `/qa-plan`.
> **PROMPT 761 Polish→Release gate-check `FAIL`**: preserved unchanged at
> `production/gate-checks/gate-polish-release-2026-05-12.md`.

Concise rough-edge readability observations for the friend-game route, gathered
by reading the Sprint 10 evidence files, the Sprint 11 draft plan, and the
existing UX specs under `design/ux/`. Each observation is a **future story
candidate**, not an immediate implementation target. The notes are scoped to the
friend-game loop — they explicitly do **not** propose broad Standard-tier
accessibility completion, do **not** claim playtest / fun-hypothesis validation,
and do **not** close `QA-COND-0005` or `QA-COND-0006`.

---

## Non-Claims

These notes explicitly do **not** claim, close, or supersede:

- public release readiness or release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005` remains
  accepted-risk friend-game scope)
- playtest / fun-hypothesis validation (`QA-COND-0006` remains accepted-risk /
  deferred)
- full playable-client manual QA
- full manual / browser two-client GAME_OVER route (`S8-QA-001-W1` remains
  OPEN)
- final-art / asset-production completion (`PAW-TD-*-a` accept-risk on
  placeholder PNGs remains in place across PAW-002..PAW-006)
- Sprint 11 activation
- closure of any existing Sprint 10 carry or any Sprint 11 row

---

## How to Read This File

| Column | Meaning |
|---|---|
| `Observation` | Rough-edge readability behaviour observed (or inferred from evidence on `main`). |
| `Why it matters` | Friend-game-loop impact. Scope is friend-game-lite, not Standard-tier accessibility. |
| `Candidate story` | Future-story slug suggestion. **Not yet filed.** Some align with already-named Sprint 11 backlog rows (see `production/sprints/sprint-11.md` "Wider Sprint 11 Backlog"); those are flagged. |
| `Disposition` | `future-story-candidate` (default) — needs its own ticket to act. `already-tracked` — backlog row exists; this is a cross-reference. `accepted-risk-friend-game` — explicitly out of scope. |

No row in this file authorises immediate implementation. A separate prompt with
its own story file + `/story-readiness` is required before any change lands.

---

## Route 1 — Lobby

Existing UX: `design/ux/main-menu.md`, `design/ux/class-picker.md`. Visual
chrome wiring landed at `S10-POLISH-003` / PAW-006 with placeholder portrait
PNGs.

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| "Confirming..." text does not distinguish own-confirm-ack vs waiting-opponent. | A second player joining mid-confirm sees an ambiguous label and may re-click. Friend-game-loop only — no class-lock authority change. | `S11-LOBBY-UX-CONFIRM-STATE-001` (Wave 8 backlog row already drafted in `production/sprints/sprint-11.md` Nice-to-Have). | already-tracked |
| Class-picker grid hit-targets and arrow stepping have not been UX-reviewed against final art; placeholder portraits PAW-006 remain. | Click-miss on small portrait targets in the friend-game loop. | `S11-UX-LOBBY-CLASS-PICKER` + `S11-UX-LOBBY-BUTTON-HITTARGETS` (PROMPT 685 UI clean-pass audit, listed in `production/sprints/sprint-11.md` "Wider Sprint 11 Backlog"). | already-tracked |
| Room-code chip is wired (`LOBBY_ROOM_CODE_CHIP_ASSET`) but a brief native eyeball check that the code remains legible on a 1920×1080 friend-game window has not been recorded. | Operator-side readability before sending the code to a friend. Cosmetic only. | `S11-UX-LOBBY-ROOM-CODE-EYEBALL-001` (new candidate). | future-story-candidate |
| Slot panels render for own + opponent, but the visual differentiation between an empty opponent slot and a "joining…" intermediate state has not been verified. | Friend-game host may mis-read "no opponent yet" as "opponent has joined but not yet class-confirmed". | `S11-UX-LOBBY-OPP-SLOT-DISAMBIGUATION-001` (new candidate). | future-story-candidate |

---

## Route 2 — Hand / Drag

Existing UX: `design/ux/hand-ui.md`. Drag-and-drop runtime evidence is the
subject of an in-flight Sprint 11 draft story.

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| Drag-and-drop runtime divergence (S1–S5 grey-square attribution truth-table from PROMPT 698 / 706 / 709) is still test-vs-runtime divergent; the live trace has not been captured against an actual friend-game session. | Drag is the primary gameplay input for placement; misattribution between S1 (placement OK) and S2..S5 (rejected states) directly degrades the friend-game loop. | `S11-DRAG-RUNTIME-RETEST-001` is **already authored** as Sprint 11 draft Must Have at `production/epics/hand-ui/story-018-drag-runtime-retest.md` (PROMPT 766). | already-tracked |
| Drag-cancel on empty cell vs invalid cell vs valid-but-occupied cell uses the same grey-square feedback today, per PROMPT 709 transcript. | Player cannot distinguish "this lane is full" from "you can't place here at all" without re-reading the HUD. | `S11-HU-DRAG-FEEDBACK-DIFFERENTIATION-001` (new candidate; conditional on S11-DRAG-RUNTIME-RETEST-001 outcome). | future-story-candidate |
| Hand UI staged disclosure for placement gating (per `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md`) is Standard-tier accessibility scaffolding, not friend-game-required. | Scope guard. | n/a — explicitly **not** a friend-game readability candidate; Standard-tier work stays under `QA-COND-0005`. | accepted-risk-friend-game |

---

## Route 3 — Draft Grid (DRAFT_INITIAL)

Existing UX: covered by `design/ux/shop-auction-ui.md` for DRAFT_INITIAL ↔
DRAFT_SHOP ↔ DRAFT_AUCTION transitions; no dedicated `draft-grid.md` exists.

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| Draft grid is not visually centered as a modal — it occupies the shop/auction panel region instead. PROMPT 685 UI clean-pass audit flagged this. | First-impression readability on session entry; players miss that the grid is the active interaction surface. | `S11-UX-DRAFT-GRID-CENTERED-MODAL` (PROMPT 685 UI clean-pass; backlog row in `production/sprints/sprint-11.md`). | already-tracked |
| `S2CObjectiveIdentities` arrives at DRAFT_INITIAL via unicast (ADR-001). The 4-objective-overlay clear timing relative to the draft grid render has been integration-tested (`production/qa/evidence/shop-auction-ui-draft-initial-clear-objective-overlay-2026-05-05.md`) but not eyeballed against a real friend-game session for visible flicker. | A perceptible flicker at session start hurts first-impression readability even though the test passes. | `S11-DRAFT-INITIAL-OVERLAY-EYEBALL-001` (new candidate). | future-story-candidate |

---

## Route 4 — Shop (DRAFT_SHOP)

Existing UX: `design/ux/shop-auction-ui.md`. Panel chrome wired at
`S10-POLISH-002` via `SHOP_PANEL_CHROME_ASSET`.

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| Auction panel root currently reuses `SHOP_PANEL_CHROME_ASSET` as a placeholder. `PAW-TD-003-a` is accept-risk for friend-game scope, but visual ambiguity between SHOP and AUCTION phases is a readability hit. | Player on first run may not realise the phase has flipped because the panel chrome looks identical. | `S11-UX-AUCTION-FEATURED-CARD` (PROMPT 685 audit row) partially covers this; a narrower `S11-UX-AUCTION-CHROME-DIFFERENTIATION-001` may also be needed. | already-tracked + future-story-candidate (narrower split) |
| Shop slot wells use placeholder PAW-003 PNGs; hover/affordance state has not been UX-reviewed against final art. | Card-buy mis-clicks; cosmetic affordance gap. | `S11-UX-SHOP-SLOT-AFFORDANCE-001` (new candidate). | future-story-candidate |
| Inline gold display in the shop (per `production/qa/evidence/economy-auction-inline-gold-evidence.md` and `hud_economy_auction_inline_gold_test`) is wired, but the read order (player sees gold-cost → gold-balance → buy-affordance) is not annotated in `design/ux/shop-auction-ui.md`. | Friend-game-loop readability of "can I afford this card right now?". | `S11-UX-SHOP-INLINE-GOLD-READ-ORDER-001` (new candidate). | future-story-candidate |

---

## Route 5 — Auction (DRAFT_AUCTION)

Existing UX: `design/ux/shop-auction-ui.md`. Bid-target focus integration test
exists (`shop-auction-ui-auction-bid-target-focus-2026-05-05.md`).

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| Featured (auction-up) card layout is not yet differentiated from shop slot well chrome — placeholder asset reuse means the auctioned card does not pop visually. | Bidding decision speed under the 30s `DraftAuction` timer is gated on instant card recognition. | `S11-UX-AUCTION-FEATURED-CARD` (PROMPT 685 UI clean-pass audit; backlog row in `production/sprints/sprint-11.md`). | already-tracked |
| Free-gold (interest / refunded-bid) counters are wired but their on-screen placement during auction has not been UX-reviewed for proximity to the bid-button cluster. | Player wants to read "do I still have gold to outbid?" in a single saccade; current layout may force two. | `S11-UX-AUCTION-FREE-GOLD-COUNTERS` (PROMPT 685 audit row). | already-tracked |
| Auction settlement transition (`shop-auction-ui-settlement-transition-evidence.md`) is integration-covered but the visual hand-off from "winner highlight" to "card lands in winner's hand" has not been eyeballed against an actual two-client friend-game session. | First-time spectator clarity in friend-game spectate mode. | `S11-UX-AUCTION-SETTLEMENT-VISUAL-EYEBALL-001` (new candidate; conditional on `S8-QA-001-W1` two-client route progress). | future-story-candidate |

---

## Route 6 — Board (Placement + Resolution)

Existing UX: `design/ux/hud.md` (RESOLUTION dim) + ad-hoc spec from PROMPT 685
flagged the need for a dedicated `design/ux/board-rendering-spec.md`.

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| Board rendering does not yet have a single spec document; sprite z-order, ghost-preview, and snapshot-spawn behaviour are spread across multiple stories (board-rendering-performance, ghost-preview-bridge, status-icons). | New contributors and reviewers can't read a single-source board-render rule sheet. | `S11-UX-BOARD-RENDERING-SPEC` (PROMPT 685 audit row). | already-tracked |
| Status icon drift on units (per `board_rendering_status_icons_test` should-panic disposition) is currently a fixture-debt story (`S11-TD-FIXTURE-D-RESIDUALS-001`), but the underlying readability question — *which status icon means what to the player?* — has not been captured anywhere player-facing. | Friend-game players misread status icons as part of the unit silhouette. | `S11-UX-BOARD-STATUS-ICON-LEGEND-001` (new candidate; can be a one-paragraph addition to `design/ux/hud.md` rather than a new doc). | future-story-candidate |
| Ghost-preview opacity / colour during drag has not been eyeballed against the final board chrome (PAW-005 placeholder sprites still in use). | Placement clarity. | `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` (new candidate). | future-story-candidate |

---

## Route 7 — HUD / Timer

Existing UX: `design/ux/hud.md`. RESOLUTION dim overlay at `α = 0.45` landed
under `S10-POLISH-001`.

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| HUD phase timer bar is integration-tested (`hud_phase_timer_bar_test` 4/4 PASS at `112ac83`) but has not been eyeballed against a live `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s session. Smoke retry-7 Warning W2 explicitly carries this. | Cosmetic verification before any public-facing demo. | `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 11 draft Should-Have; backlog row in `production/sprints/sprint-11.md`). | already-tracked |
| HUD top-strip / bottom-strip / opponent figurine layout audit from PROMPT 685 is unsplit into discrete stories. | Top-strip is where gold / mana / phase live; layout drift directly hurts every-frame readability. | `S11-UX-HUD-TOP-STRIP-LAYOUT` + `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` + `S11-UX-HUD-OPP-FIGURINE` (PROMPT 685 UI clean-pass audit rows). | already-tracked |
| RESOLUTION dim overlay alpha is hard-coded at `HUD_DIM_OVERLAY_ALPHA = 0.45` (`client/src/ui/hud/mod.rs`). Final-art pass may want a tween or different value. | Out-of-scope per S10-POLISH-001 (HUD-12b explicitly forbids tween). Recorded so a future polish pass does not re-derive the constant. | n/a — accepted-risk friend-game scope; revisit when final art lands. | accepted-risk-friend-game |
| Audio timer urgency cue (`audio-timer-urgency-2026-05-08.md`) is wired but its visual counterpart (a colour shift / pulse on the timer bar near phase end) has not been speced. | Multimodal redundancy is friend-game-loop nice-to-have, not Standard-tier required. | `S11-UX-HUD-TIMER-URGENCY-VISUAL-001` (new candidate). | future-story-candidate |

---

## Route 8 — Result / Close-Out

Existing UX: `design/ux/result-screen.md`. Result screen MVP +
`result-screen-mvp-evidence.md` + `result-screen-ux-review-refresh-2026-05-07.md`
already on `main`.

| Observation | Why it matters | Candidate story | Disposition |
|---|---|---|---|
| The full manual / browser two-client GAME_OVER route has never been captured end-to-end. `S8-QA-001-W1` has been OPEN since Sprint 8 and was carried unchanged through Sprints 9 and 10. | Result-screen readability evidence is integration-test-only today; no live two-client photograph exists. | `S8-QA-001-W1` — **already tracked**; resolution path is `production/qa/evidence/manual-friend-game-evidence-runbook.md` (human operator). | already-tracked |
| Result acknowledgement / cleanup handshake (`result-acknowledgement-cleanup-handshake-evidence.md`) is integration-covered but the visible client-side "returning to lobby…" intermediate state has not been UX-reviewed. | Friend-game host expects a clear "rematch?" affordance; current path is implicit. | `S11-UX-RESULT-RETURN-TO-LOBBY-001` (new candidate; conditional on `S8-QA-001-W1` progress). | future-story-candidate |
| `QA-COND-0007` resolution replay readability (`qa-cond-0007-resolution-replay-readability-2026-05-06.md`) closed under Sprint 9 but only covers the replay scrubber, not the post-game stat summary readability. | Scope guard — replay UX is one slice; result-screen stat summary is another. | n/a — replay covered; stat-summary readability is folded into the existing `design/ux/result-screen.md` and any change is a Sprint 11+ UX story, not a separate candidate here. | accepted-risk-friend-game |

---

## Cross-Route Notes

- **All "new candidate" rows above are paperwork only.** No story file exists
  yet for any of them; before `/dev-story` can begin on any, a story file plus
  `/story-readiness` is required in a separate prompt.
- **PROMPT 685 UI clean-pass audit** already drafted a coherent 8-story
  milestone of UX stories (listed in `production/sprints/sprint-11.md` "Wider
  Sprint 11 Backlog"). The `already-tracked` rows above are cross-references
  to that audit — they do not duplicate it.
- **Final art is out of scope.** Every observation that begins with
  "placeholder PNG…" is friend-game accept-risk under `PAW-TD-*-a`. Replacing
  placeholders is a separate art-production initiative, not a UX readability
  fix.

---

## Authoring Disposition (PROMPT 772)

PROMPT 772 authored this notes file under draft Sprint 11 story
`S11-ROUTE-READABILITY-CARRY-001` (carried from deferred Sprint 10 nice-to-have
`S10-N2` per PROMPT 763 close-out and PROMPT 764 Sprint 11 draft plan).

PROMPT 772 did **not**:

- run `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/qa-plan`, or any implementation
- modify production code under `client/`, `server/`, `shared/`, or `tests/`
- mutate `production/sprint-status.yaml`,
  `production/sprints/sprint-11.md`, or `production/stage.txt`
- modify `.octogent/`, `.gitignore`, `.claude/settings.json`, `reports/`, or
  `.claude/scheduled_tasks.lock`
- activate Sprint 11
- flip the Sprint 11 `S11-ROUTE-READABILITY-CARRY-001` row to `done` (Sprint
  11 activation-time decision)
- propose Standard-tier accessibility completion
- claim closure of `QA-COND-0005`, `QA-COND-0006`, `S8-QA-001-W1`, or any
  other carried condition
- claim playtest / fun-hypothesis validation
- claim public-release readiness, release-candidate readiness, full-game
  completion, full playable-client manual QA, or final-art / asset-production
  completion

Files touched by PROMPT 772:
`production/qa/evidence/sprint-10-route-readability-notes.md` (NEW — this
file), `production/session-state/active.md` (banner update),
`production/session-state/codex-orchestrator-state.md` (disposition section).
