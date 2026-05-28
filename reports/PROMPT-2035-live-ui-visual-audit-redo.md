# PROMPT-2035 — Live UI Visual Audit Redo

**Date**: 2026-05-28
**Source of truth**: `origin/main@8f7d3502`
**Scope**: Audit / report only. No game code edited.
**Baseline inputs**:
- `production/qa/bugs/current-unplayable-bug-register-2026-05-28.md`
- `reports/PROMPT-2024-forensic-evidence-inventory-and-run-selection.md`
- `reports/PROMPT-2025-snapshot-log-gamestate-correlation-audit.md`
- `reports/PROMPT-2026-visible-screen-screenshot-visual-bug-audit.md`
- `reports/PROMPT-2027-autoplay-input-click-target-forensic-audit.md`
- `reports/PROMPT-2028-player-flow-unplayable-bug-classification.md`
- `reports/PROMPT-2029-qa-evidence-tools-truthfulness-audit.md`
- `reports/PROMPT-2033-server-board-gameover-vacuous-flow-p0-repair.md`

---

## Method

For each user-observed UI surface I checked:

1. **Live autoplay screenshots** under the three audited runs of 2026-05-28
   (`20260528-051148-Z`, `20260528-063609-Z`, `20260528-090613-Z`) — these are
   the only artifacts captured from a real running Bevy client in the current
   build lineage.
2. **Driver timeline** (`driver-timeline.jsonl`) at the matching tick to
   confirm what phase the *client state machine* believed it was in when the
   screenshot was taken.
3. **Server snapshots** at the same checkpoint to confirm whether the server
   advanced the phase the client failed to render.
4. **Older baseline captures** under `production/qa/evidence/captures/**` only
   as supplementary "this used to exist" evidence, never as proof of current
   visible state (per PROMPT 2029 T-030: those captures are Chrome / harness
   captures, not live integrated Bevy client sessions).

Per the prompt rule "If no artifact exists for a claimed surface, mark it as
coverage missing, not PASS", every in-game surface (shop, auction, placement
board, hand fan, drag/drop overlay, combat/resolution, result) is **coverage
missing on the live build** — because the autoplay client never leaves Lobby.
PROMPT 2026 confirms this for all 15 checkpoints across all three runs:
`client_state_label:"Lobby"` for 262/262 ticks.

---

## User-Observed Bug Map

| Bug ID | User-reported symptom | Verdict | Artifact path(s) | Visible symptom in artifact | Likely owner / module | Repair priority |
|---|---|---|---|---|---|---|
| UB-01 | Drag / drop broken | **Coverage missing on live build** — also blocked by BUG-01 / P0-002 | autoplay `…/090613-Z/screenshots/{000048,000052}.png`; `driver-timeline.jsonl` tick 48/52 | Checkpoint `placement-dragged` and `placement-submitted` capture the **Lobby** class picker — no board, no hand, no drag overlay. Drag click dispatched at logical 1280×720 coords but only Lobby is rendered. | `src/ui/placement/**`, `src/ui/interaction_state/**` (drag pickup/drop) — blocked behind client-phase-sync repair (PROMPT 2030) | P0 (blocked by P0-002) |
| UB-02 | Hand fan spacing wrong | **Coverage missing on live build** | Same autoplay runs — no checkpoint ever captures the hand fan; only older Chrome harness captures `production/qa/evidence/captures/qa-cond-0007-hand-ui/*.png` exist | Old harness PNGs show a hand fan rendered through a Chrome harness, not the integrated Bevy game (T-030/T-023). No live fan-spacing screenshot exists in current build. | `src/ui/hand/**` + bevy_ui flex / fan layout — verify after BUG-01 lifted | P1 (cannot diagnose until in-game UI renders) |
| UB-03 | Valid-cell highlighting missing | **Coverage missing on live build** | Older Chrome harness `…/hand-ui-placement-staged-disclosure/{03-lane-cell-target-guidance,04-valid-target-highlight}.png` | Highlight pattern exists in older harness capture only; no live Bevy capture proves it renders today | `src/ui/placement/board_hittest_*` + `src/ui/board/overlay_*` | P1 (post BUG-01) |
| UB-04 | Invalid placement feedback missing | **Coverage missing on live build** | Older `…/hand-ui-placement-staged-disclosure/07-invalid-submit.png` + `…/qa-cond-0007-hand-ui/06-invalid-submit-inline-correction.png` | Both are non-live harness captures. Live autoplay never reaches a placement-rejection state in a visible client. PROMPT 1468 landed a rejection-recovery UX but no current live screenshot verifies it. | `src/ui/placement/rejection_*` (PROMPT 1468 owner) | P1 (post BUG-01) |
| UB-05 | Card placeholder art (cards show black face) | **Confirmed visually** (Lobby class cards) — extends user complaint | autoplay `…/090613-Z/screenshots/000037.png`; `…/090613-Z/win32_tick_000185.png`; `…/090613-Z/win32_tick_000259.png` | Reproduces PROMPT-2026 BUG-02 / V1-001 / V1-002: all 7 class buttons (Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida, Neutral) render as a solid black rectangle; only the colored element gem in the top-right corner is drawn. Selected-class preview card is also black. Neutral card half-clipped at right edge (V1-002). | Class card art asset binding — `src/ui/lobby/class_picker_*`; asset path or `bevy_asset_loader` group for class-card art. Likely a `Handle<Image>` falling back to a default (= black) because the proxy art is unbound. Cross-check PROMPT-1933 / PROMPT-1957 (auction tier border asset binding refresh chain). | **P0 visual** — only screen the player can see |
| UB-06 | Missing card stats on cards | **Coverage missing on live build** | None on live; older harness shows stat labels in `qa-cond-0007-hand-ui/*.png` | No live in-game card surface has been captured this build. Lobby class cards show only element gem + class name — no stat block by design. | `src/ui/card/stat_strip_*` + draft slot wiring; tied to T-010 / T-011 / T-012 (snapshot missing class/rarity fields) | P1 (post BUG-01) |
| UB-07 | Broken shop visuals | **Coverage missing on live build** — checkpoint reached but Lobby still rendered | autoplay `…/090613-Z/screenshots/{000022 shop-loaded, 000026 shop-slot-clicked}.png` | Both shop checkpoints are pixel-identical to `000000 lobby-loaded`. The "shop visual" complaint cannot be evaluated because the shop never renders. PROMPT 2025 confirms server is in `DraftInitial`/`DraftShop` at those ticks while client is in Lobby. | Client phase sync (PROMPT 2030 P0); after that, `src/ui/shop/**` + `src/ui/draft/**` | P0 (blocked by P0-002) |
| UB-08 | Board unit rendering absent | **Confirmed at server level + coverage missing visually** | autoplay `…/090613-Z/screenshots/{000054 resolution-started,000055 resolution-complete,000057 vs-bot-post-resolution}.png`; PROMPT 2025 snapshots; PROMPT 2033 root-cause | All resolution checkpoints render Lobby. Server snapshots confirm `per_player_minions` empty + board counts zero for the entire run (P0-007). PROMPT 2033 root-caused this to the empty-board cascade: no client InSession + bot hand/placement empty → no submitted units. | `src/ui/board/unit_render_*` + upstream placement-submit pipeline. Repairs queued: PROMPT 2030 (client phase), PROMPT 2031 (draft hand awarding), PROMPT 2032 (bot placement failsafe). | P0 (cascade) |
| UB-09 | Combat / resolution presentation absent | **Confirmed at server level + coverage missing visually** | Same checkpoints as UB-08; PROMPT 2025 snapshots show resolution phases entering and exiting in ms with empty board | Resolution screen never appears; checkpoint `resolution-started` captures Lobby. PROMPT 2033 shows resolution is instant because board is empty (no combat to simulate). | `src/ui/resolution/replay_*` (PROMPT 1521/1527/1528 owners); blocked behind board-units cascade | P0 (cascade) |
| UB-10 | Global UI anchoring / layout failures | **Confirmed at Lobby + extrapolated for in-game** | autoplay `…/090613-Z/win32_tick_000185.png` (1280×1076 native), `…/090613-Z/win32_tick_000259.png`; older `production/qa/evidence/captures/board-rendering-baseline-1920x1080.png`; older `…/shop-auction-ui-auction-bid-target-focus/sau-011-bidding-1366x768.png` | Reproduces BUG-07 / V1-004: at 1280×1076 the Lobby panel is ~790×870 px centered, leaving ~245 px of dark margin per side. Class-picker row overflows (BUG-06 / V1-002). Older board baseline shows the 5-lane board rendered as a ~400×310 island on 1920×1080 with no HUD chrome (BUG-08 baseline / V1-005). Older auction screen lacks card art and slot context (BUG-09 baseline / V1-006). Window resize mid-run (T-005): logical height 720 → 1076 native 759 → 1115 detected by PROMPT 1880 drift guard. | `src/ui/layout/anchor_*`, root `bevy_ui` `Style { ... }` flex containers, viewport scaling. Cross-system. Owners: `ui-programmer` + `liv-bevy-018` (Required Components / Style migration). | P1 visual; P0 for window-drift behavior already mitigated by PROMPT 1880 |

### Additional confirmed visual issues already flagged in baseline register

| Bug ID (baseline) | Title | Status this audit |
|---|---|---|
| V1-003 | Lobby header separator glyphs render as tofu boxes (`▢`) | Confirmed reproducing in `…/090613-Z/win32_tick_000185.png` header `Connected as player 9 ▢ Room: ---- ▢ Players: 0/1` |
| V1-007 / BUG-10 | Room code input renders as debug placeholder `Type room code: -------- - idle` | Confirmed in `…/090613-Z/screenshots/000000.png` and `000007.png` |
| V1-008 / BUG-11 | QA Snapshot button visible top-right | Confirmed in every screenshot — acceptable iff `CCGS_QA_SNAPSHOT=1` is intentional (advisory only) |
| BUG-03 / P1-002, P1-003 | `Room: ----` and `Players: 0/1` never update post-create-room or post-bot-join | Confirmed in all 15 checkpoints |
| BUG-04 / P1-004 | `not confirmed` never clears after class-confirmed checkpoint | Confirmed across screenshots `000020`+ |

---

## Mismatch / falsifiability findings

- **Checkpoint label vs visible state mismatch**: 9 of 15 checkpoints in the
  latest live run carry in-game labels (`shop-loaded`, `auction-loaded`,
  `placement-loaded`, `placement-dragged`, `placement-submitted`,
  `resolution-started`, `resolution-complete`, `vs-bot-post-resolution`,
  `auction-ready`) while the rendered screen is still Lobby. Per task rule
  this is NOT a PASS for those surfaces — it is **coverage missing for the
  named surface** plus a confirmed checkpoint-truthfulness bug (T-021).
- **Server vs client mismatch**: PROMPT 2025 server snapshots advance
  `DraftInitial → Placement → Resolution → DraftShop → Placement → Resolution
  → GameOver` while client `client_state_label` stays `Lobby` for 262/262
  ticks (PROMPT 2026). The user-observed "in-game UI is broken" is therefore
  *upstream* of the in-game UI: the in-game UI is never instantiated.
- **Old harness captures are not live proof**: the only artifacts showing a
  hand fan, valid-cell highlight, invalid-submit feedback, card stats, or
  shop card grid are Chrome harness / `qa-cond-0007` PNGs from earlier builds
  (T-030 / T-023 / T-032). They cannot confirm the live in-game UI for
  drag/drop, hand fan spacing, valid-cell highlight, invalid feedback, card
  stats, or shop visuals on `origin/main@8f7d3502`.

---

## Repair priority summary

| Priority | Items | Rationale |
|---|---|---|
| **P0 — blocks all visual diagnosis** | UB-01, UB-07, UB-08, UB-09, plus BUG-01 (P0-002). Repair queued under PROMPT 2030 (client phase sync), PROMPT 2031 (server draft / hand awarding), PROMPT 2032 (bot placement failsafe). | Until the client leaves Lobby, none of UB-01..09 can be verified or refuted on the live build. |
| **P0 visual** | UB-05 (V1-001 black class card art); V1-002 Neutral card clipping. | Class-select is the only screen a real player can currently see; black faces make the only visible screen unusable. |
| **P1 visual** | UB-10 / BUG-07 (window margins, anchoring), V1-003 separator tofu, V1-007 room-code input styling, BUG-03 / BUG-04 (lobby state never updates). | Visible today, all on the Lobby surface; non-blocking but degrade the only visible screen. |
| **P1 (post BUG-01)** | UB-02, UB-03, UB-04, UB-06 — verify after client phase sync lands; current verdict is coverage missing, not PASS. | Need a real in-game capture before any in-game UI claim can be PASS/FAIL. |
| **Evidence taxonomy** | T-020..T-033 (per PROMPT 2029): semantic validators, phase-gated recipes, real-client-only PASS gating, treat `NEEDS_HUMAN_GUI` as blocking. | Without these, the next refresh of this audit will keep reading "checkpoint=shop-loaded" as PASS while the user still sees Lobby. |

---

## Conclusion

The user's catastrophic-UI report is **substantially correct**, but the
mechanism is upstream of the in-game UI: the client state machine never
transitions to `InSession`, so the in-game UI (drag/drop, hand fan,
valid-cell highlight, invalid-placement feedback, card stats, shop, board
units, combat/resolution) is *never instantiated*. The five live in-game
surfaces named in the user complaint cannot be classified PASS or FAIL on the
current build because no artifact exists — they are coverage-missing pending
PROMPT 2030 / 2031 / 2032 landing.

What can be confirmed today against live screenshots:

- Client stuck in Lobby for the entire game (BUG-01 / P0-001..P0-003).
- Class card art black for all classes (UB-05 / V1-001) — only visible screen
  has broken art.
- Neutral class card clipped (V1-002).
- Header separator glyphs render as tofu (V1-003).
- `Room: ----`, `Players: 0/1`, `not confirmed` never update (BUG-03, BUG-04,
  P1-002..P1-004).
- Large dark margins at 1280×1076 (BUG-07 / V1-004).
- Mid-run window resize 720 → 1076 logical (T-005) — mitigated by PROMPT 1880
  drift guard, requires a fresh guarded run for re-verification.

Recommended next moves (no code touched here):

1. Land PROMPT 2030 / 2031 / 2032 then **re-run this audit** on the resulting
   build with the same checkpoint set so UB-01..04 and UB-06..09 can be moved
   off coverage-missing.
2. Open a P0-visual repair PROMPT for UB-05 (class card art binding) —
   independent of the InSession cascade, fixable today, and the only screen
   the player can currently see.
3. Wire the evidence-taxonomy guardrails from PROMPT 2029 (T-020..T-033) so
   future "checkpoint reached" cannot count as visual proof when
   `client_state_label != phase_label`.

No game code, test, sprint, or session-state files were modified by this
audit. Only this report was added under `reports/`.

---

2035: LIVE-UI-VISUAL-AUDIT-REDO: SHIPPED
