# HUD Design

> **Status**: Complete — pending /ux-review
> **Author**: user + ux-designer
> **Last Updated**: 2026-04-29
> **Template**: HUD Design
> **Input Methods**: Mouse + Keyboard (primary: mouse click). No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard (see `design/accessibility-requirements.md`)
> **Art Direction**: `design/art/art-bible.md` Section 7

---

## HUD Philosophy

**Perimeter-ring, phase-adaptive.**

All persistent decision-relevant information lives at screen edges. The board center is never obstructed during play. Phase-specific panels (auction, shop) occupy fixed screen slots and fade in/out on phase transition — they never slide in from off-screen.

This philosophy serves two pillars simultaneously:
- **No idle spectating**: all resource data is always readable, so even during RESOLUTION there is live information to process (opponent gold, objective dot changes, board state).
- **Auction as signature**: the auction panel's full-screen prominence is architecturally guaranteed by the philosophy — it is not a modal overlay, it is the primary state.

**HUD density by phase:**

| Phase | HUD density | Note |
|---|---|---|
| LOBBY | Low | Class select UI dominates; persistent elements minimal |
| DRAFT_INITIAL | Medium | All core resources visible; card selection UI prominent |
| DRAFT_AUCTION | **Auction-mode** | Auction panel replaces board; core resources accessible at edges |
| DRAFT_SHOP | Medium | Core resources + shop visible |
| PLACEMENT | **High** | All resources visible; placement timer prominent; opponent half fogged |
| RESOLUTION | Low | HUD dims to 70%; board is hero; no player input |
| GAME_OVER | Reveal | Objective identities exposed; win/loss state prominent |

---

## Information Architecture

### Full Information Inventory

| # | Information | Source System | Update trigger |
|---|---|---|---|
| 1 | Player gold (own) | Economy | Any gold change event |
| 2 | Opponent gold | Economy / Network | `S2CGoldBroadcast` |
| 3 | Current mana | Economy | DRAFT entry (reset), card spend |
| 4 | Reserve mana | Economy | Card spend, Gelure, Prism reward |
| 5 | Mana cap | Economy | Game start, fake objective destroyed |
| 6 | Phase label | RSM | `S2CPhaseChanged` |
| 7 | Round number | RSM | `S2CPhaseChanged` |
| 8 | Phase timer | RSM | `S2CPhaseChanged` (duration), local countdown |
| 9 | Own objective dots × 5 (hp + real/fake) | Objective System | `ObjectiveHp` replication, `S2CObjectiveIdentities` |
| 10 | Opponent objective dots × 5 (hp only) | Objective System | `ObjectiveHp` replication |
| 11 | Own class figurine + HP | Game Session / RSM | `SessionConfig`, RSM events |
| 12 | Opponent class figurine + HP | Game Session / RSM | `SessionConfig`, RSM events |
| 13 | Interest threshold indicator | Economy | Hover on gold counter |
| 14 | Spawn range highlight | Board/Lane | PLACEMENT phase entry, `spawn_range` state |
| 15 | Hand card count | Card Pool / Network | `S2CCardAcquired`, card plays |

### Categorization

| Category | Items |
|---|---|
| **Must Show** (always visible, all phases) | 1 — Own gold · 2 — Opponent gold · 3 — Current mana · 4 — Reserve mana · 5 — Mana cap · 6 — Phase label · 7 — Round number · 8 — Phase timer · 9 — Own objective dots · 10 — Opponent objective dots · 11 — Own class figurine · 12 — Opponent class figurine · 15 — Hand card count |
| **Contextual** (visible only when relevant) | 13 — Interest threshold indicator (on hover over own gold counter) · 14 — Spawn range highlight (PLACEMENT phase only, on board cells) |
| **On Demand** | *(none — all information is passively available)* |
| **Hidden** | *(none — all strategic information has a visual representation)* |

**Must Show count: 13 items.** This is intentionally information-dense. The "No idle spectating" pillar means there is always something to read.

---

## Layout Zones

Perimeter-ring layout. Board occupies center. All HUD anchored to screen edges.

```
┌─────────────────────────────────────────────────────────────────┐
│ [OWN CLASS FIGURINE]    [OBJECTIVE DOTS ×10]    [OPP CLASS FIGURINE] │  ← TOP STRIP
│                       [PHASE LABEL · RND#]                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  [BOARD — 5 lanes × 8 cells]                                     │  ← CENTER
│  (never obstructed by persistent HUD)                            │
│                                                                   │
├──────────────────────────────────┬──────────────────────────────┤
│ OWN GOLD  OPP GOLD  PHASE TIMER  │  MANA BAR  RESERVE ◆  CAP   │  ← BOTTOM STRIP
│                                  │                               │
│ [HAND CARDS — scrolling row]     │                               │
└──────────────────────────────────┴──────────────────────────────┘
```

**Zone definitions:**

| Zone | Contents | Always visible? |
|---|---|---|
| Top-left corner | Own class figurine | Yes |
| Top-center strip | 10 objective dots (own 5 left, opponent 5 right), phase label, round number | Yes |
| Top-right corner | Opponent class figurine | Yes |
| Bottom-left cluster | Own gold, opponent gold, phase timer | Yes |
| Bottom-right cluster | Current mana bar, reserve mana diamond, mana cap label | Yes |
| Bottom strip (full width) | Hand cards row (left-anchored, max 10) | Yes |
| Board surface | Spawn range cell highlights (PLACEMENT only), placement fog (PLACEMENT only) | Contextual |

**Auction-mode exception:** During DRAFT_AUCTION, the board zone is replaced by the auction panel. The top strip (objectives, phase label) and figurines remain. Bottom resources remain accessible at reduced opacity.

---

## HUD Elements

### Own Gold Counter
- **Category**: Must Show · **Position**: Bottom-left, leftmost element
- **Content**: Current gold value as large numeral (e.g., `7`)
- **Visual**: Arcane Gold `#F5C842` heavy numeral, Void outline. Coin icon left of number.
- **Update**: Count-up/count-down animation on change (max 400ms). No animation on phase resets (gold doesn't reset).
- **Hover behavior**: Shows interest threshold tooltip — "Hold `X` gold by round end to earn bonus (+1 at 5g, +2 at 10g)"
- **Never occluded** — always at HUD layer, never covered by panels.
- **Accessibility**: Shape (coin icon) accompanies color. Min 40px numeral height.

### Opponent Gold Counter
- **Category**: Must Show · **Position**: Bottom-left, right of own gold
- **Content**: Opponent's current gold value
- **Visual**: Same size and weight as own gold — equal visual importance (art bible §7.3 conflict resolution: opponent gold = equal weight during auction)
- **Label**: Small "OPP" or opponent class icon above the number for disambiguation
- **Update**: Animates on `S2CGoldBroadcast`

### Current Mana Bar
- **Category**: Must Show · **Position**: Bottom-right cluster, leftmost
- **Content**: Current mana as a segmented bar (each segment = 1 mana). Filled = available. Empty = spent.
- **Visual**: Teal `#2AA8C4` filled segments, Ink Blue empty segments, Void outline. Horizontal bar, left-to-right fill.
- **Update**: Segments drain instantly on card play. Refills via count-up animation at DRAFT entry (mana ramp).
- **Mana cap**: Number label above bar showing max (e.g., `10`). Updates with +1 animation when mana cap increases.
- **Accessibility**: Bar shape independent of color. Segment count = numeric value label above bar.

### Reserve Mana Diamond
- **Category**: Must Show · **Position**: Bottom-right cluster, right of mana bar
- **Content**: Reserve mana as a numeral inside a diamond shape
- **Visual**: Blue gradient diamond with soft Prism White inner glow (art bible §7.5). Reserve numeral inside in heavy white text.
- **Distinct container**: Diamond shape (not bar) — enforces "these are different resources" without color alone (accessibility requirement §cognitive)
- **Persist indicator**: Small loop/cycle glyph below diamond confirming "carries to next round"
- **Update**: Pulses on gain (Gelure, Prism reward). Drains instantly on spend.
- **No cap indicator**: Reserve is uncapped by design. No max label.

### Phase Label + Round Number
- **Category**: Must Show · **Position**: Top-center, below objective dots
- **Content**: Phase name (e.g., `PLACEMENT`) and round (e.g., `Round 4`)
- **Visual**: Ivory `#F7F0DC` Regular weight, centered. Phase name larger than round number.
- **Always present** — never hidden, never animation-only. Persistent text label is required (accessibility §cognitive).
- **Phase transition**: Brief fade-in 80ms · hold 600ms · fade-out 80ms. New label fades in.

### Phase Timer
- **Category**: Must Show · **Position**: Bottom-left cluster, right of gold counters
- **Content**: Countdown in seconds (e.g., `10`, `30`, `45`)
- **Visual**: Heavy numeral. Color changes by urgency:
  - > 15s: Ivory (calm)
  - 6–15s: Auction Amber (attention)
  - ≤ 5s: Crimson-Amber (urgent)
- **PLACEMENT timer specifics**: Hard server-owned deadline. Base duration is 10s for standard PLACEMENT; the RSM may send a longer effective duration when the frozen room/session timer multiplier applies. Large, prominent. Semi-opaque background behind numeral (never rendered directly over animated board cells — accessibility §motor).
- **PLACEMENT timer accessibility**: Multiplayer Standard tier supports neutral room/session multiplier values 1x, 1.5x, 2x, and 3x per ADR-023 and `design/accessibility-requirements.md`. HUD displays the effective server timer, not a local calculation.
- **When no timer**: RESOLUTION shows no countdown (RSM drives resolution duration; no player deadline). Timer element hides.

### Objective Dots
- **Category**: Must Show · **Position**: Top-center strip, flanking the phase label
- **Own 5 dots**: Left side. Filled with own-side color + real/fake distinction (own player only sees which are real — opponent sees only hp state).
- **Opponent 5 dots**: Right side. Same visual, hp state only.
- **Dot states**:
  - Active real (own): Arcane Gold fill, solid
  - Active fake (own): Ivory fill with subtle `?` texture — only visible to owner
  - Active (opponent view): Neutral stone fill — both real and fake look identical
  - Destroyed: Dot cracks, dims to dark stone, shrinks to 80% scale — shape change on destroy
- **Dots must behave identically for real vs. fake before destruction** (art bible §9.3 prohibition). No pulse, size, or animation difference.
- **Size**: 12px circles. Pulse to 130% scale and return on objective HP loss. Full pulse on destruction.

### class figurines
- **Category**: Must Show · **Position**: Top-left (own) and top-right (opponent)
- **Content**: Class figurine art (large, 3D-style), HP counter on pedestal base
- **HP display**: Large numeral on pedestal. Updates with brief flinch animation on HP loss.
- **Scale**: ~80–120px height. Flanks the board like cornerposts.
- **Idle animation**: Slow loop per class identity (art bible §5.4).

### Hand Card Row
- **Category**: Must Show · **Position**: Bottom strip, full width, left-anchored
- **Content**: Cards in hand, 75% card scale, horizontal row
- **Hover**: Card scales to 100%, lifts 12px, full detail visible
- **Selected**: Gold outline pulse, lifts to 100% scale
- **Count**: No ghost frames for empty slots. At 10 cards: 90% spacing minimum.
- **Phase visibility**: Cards dimmed during RESOLUTION (board is hero). Full opacity all other phases.

---

## Dynamic Behaviors

### Phase Transitions
- All HUD elements that change between phases **fade in/out at their fixed positions** — never slide from off-screen.
- Phase label fades out (80ms) → new label fades in (80ms). Persistent text always present at some opacity.
- DRAFT_AUCTION: board contracts to zero scale, auction panel expands from card center (200ms ease-out). Top strip stays visible.
- PLACEMENT → RESOLUTION: hand tray dims to 50%; board becomes full focus; HUD dims to 70% overall.
- GAME_OVER: objective dots reveal (iris-open wipe per art bible §2). All hidden opponent data revealed simultaneously.

### Timer Behavior
- Phase timer visible whenever a countdown is active. Hidden during RESOLUTION (no player deadline).
- PLACEMENT timer: if extended by the server-authoritative room/session multiplier, `S2CPhaseChanged.timer_duration_ms` already contains the effective duration. The urgency color ramp stays proportional to remaining time.
- Timer stops (shows `0`) on early exit (all players submitted), does not hide.

### Gold Counter Updates
- Count-up/count-down animation over max 400ms.
- On new DRAFT income (baseline + interest): count-up from previous value to new value.
- On auction win (gold deducted): count-down from previous value.
- Counter never jumps; always animates the delta.

### Reserve Mana Changes
- Reserve gains (Gelure, Prism) trigger a pulse animation on the diamond (150ms scale to 115%, return).
- Reserve spend: diamond numeral counts down, no scale change (instant spend, no ceremony — it's a cost, not a reward).

### Mana Cap Increase
- On fake objective destroyed (mana cap +1): mana bar gains a new segment with a brief gold flash animation (new segment slides in from right, 200ms).
- Mana cap label above bar updates simultaneously.

### Objective Dot Updates
- HP change: dot pulses (scale 130% → 100%, 150ms).
- Destruction: dot cracks, dims, shrinks to 80%, permanently inert. Shape change confirms state.
- GAME_OVER: fake objectives on own side reveal their `?` marker dissolving into a confirmed identity marker (iris-open wipe, 300ms per dot, staggered left-to-right).

---

## Platform & Input Variants

**Primary platform: WASM browser. Mouse + Keyboard.**

No gamepad support. No touch support (current scope). No platform variants required at this time.

**Browser-specific considerations:**
- HUD must render correctly at browser zoom levels 75%–150% (CSS zoom). Pointer targets (bid input, submit button) minimum 44×44 CSS px.
- No system-level HUD scaling — in-game UI scale setting (75%–150%) is the mechanism.
- PLACEMENT phase: keyboard focus must be trapped within the game canvas. Tab key must not escape to browser chrome during the 10-second timer.

---

## Accessibility

*Standard tier. Source: `design/accessibility-requirements.md`.*

| Requirement | How addressed |
|---|---|
| PLACEMENT timer extension (1x-3x multiplayer) | Neutral room/session timer setting from ADR-023. 10s base -> 30s at 3x. HUD displays the effective server-provided duration without player attribution. |
| Mana pools distinct by shape (not only color) | Current mana = segmented bar. Reserve mana = diamond shape. |
| Colorblind: player A/B distinction | Circle (A) vs diamond (B) on class figurines. Shape backup per art bible §4.6. |
| Colorblind: class identity | Class icon always shown alongside class color. |
| Colorblind: objective dots | Dot destruction = shape change (cracked/shrunken) not only color change. |
| Text contrast: gold counter | Arcane Gold on Ink Blue ≥ 4.5:1. Verify with contrast checker. |
| Text contrast: placement timer | Semi-opaque background behind timer numeral. Never rendered over animated board cells. |
| Text size: resource counters | Min 20px. Gold counter and auction price min 40px. |
| Interest threshold | Triggered by hover (pointer event), not revealed passively — low motor barrier. |
| Screen reader | Not addressed in current scope. Menu screen reader is a future consideration. |
| PLACEMENT focus trap | Keyboard focus locked to game canvas during PLACEMENT phase (browser zoom safety). |
| Motion reduction | Phase-transition sweep animation can be reduced. Bid pulse can be reduced. Toggleable in settings. |

---

## Acceptance Criteria

- [ ] Own gold counter is visible in all 7 phases and is never occluded by any panel or animation
- [ ] Opponent gold counter renders at the same font size and weight as own gold counter
- [ ] Current mana bar and reserve mana diamond are distinct shapes — a colorblind player can tell them apart without color alone
- [ ] Interest threshold tooltip appears on hover over own gold counter and disappears on mouse-out
- [ ] Phase label text updates on every `S2CPhaseChanged` event with no flash or layout shift
- [ ] Phase timer counts down correctly from the `S2CPhaseChanged.timer_duration_ms` value for DRAFT_INITIAL, DRAFT_SHOP, and PLACEMENT, including extended PLACEMENT durations.
- [ ] PLACEMENT timer stops (shows `0`) when all players submit before timer expires — does not continue counting down
- [ ] Objective dots update immediately on `ObjectiveHp` replication change with pulse animation
- [ ] Destroyed objective dot shows a visually distinct state (cracked/shrunken/dimmed) from an intact dot, detectable without color alone
- [ ] Real vs fake objective dots on own side are visually identical before destruction (no animation, size, or pulse difference)
- [ ] HUD dims to 70% opacity during RESOLUTION phase and returns to full opacity at next DRAFT entry
- [ ] Auction panel replaces board during DRAFT_AUCTION; top strip (objectives, phase label) and class figurines remain visible
- [ ] PLACEMENT timer multiplier affects HUD only through the server-provided `S2CPhaseChanged.timer_duration_ms`; HUD does not multiply local Settings values, and urgency color ramp remains proportional
- [ ] Keyboard focus is trapped within the game canvas during PLACEMENT phase; Tab key does not escape to browser chrome
- [ ] All interactive HUD elements (bid input, submit placement button) have a minimum 44×44 CSS pixel click target

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-HUD-1 | Should the opponent's class figurine show the class name as a text label, or rely on the figurine art + color alone? (Relevant for players unfamiliar with Dofus/Wakfu class names) | ux-designer | Medium |
| OQ-HUD-2 | During DRAFT_AUCTION, what opacity level for the bottom resource strip — fully visible (100%) or reduced (60–70%)? Higher = easier to track gold budget while bidding; lower = more visual focus on the auction card | ux-designer | Low |
| OQ-HUD-3 | Reserve mana is uncapped. Should the diamond display show a warning state at very high values (e.g., `20+`) to hint at Garde-Temps threat? Or stay neutral? | game-designer | Low |
| OQ-HUD-4 | Should the mana cap label be shown inline with the mana bar (e.g., `7/10`) or as a separate indicator above the bar? | ux-designer | Low |
| OQ-HUD-5 | ~~Placement timer multiplier range: should multiplayer include ×0.5?~~ Resolved 2026-05-05 by ADR-023: multiplayer Standard tier is 1x-3x only; 0.5x may exist only as solo/custom/debug pace if documented elsewhere. | producer | Closed |
