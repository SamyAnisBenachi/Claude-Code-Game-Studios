# Accessibility Requirements — Lanes and Lies

> **Status**: Draft
> **Author**: ux-designer / producer
> **Last Updated**: 2026-05-05
> **Accessibility Tier Target**: Standard
> **Platform(s)**: WASM Browser (primary) · Native desktop (dev/debug target only)
> **External Standards Targeted**: WCAG 2.1 Level AA (browser UI); Game Accessibility Guidelines (basic)
> **Accessibility Consultant**: None engaged
> **Linked Documents**: `design/gdd/systems-index.md`, `design/art/art-bible.md` (Section 4.6 — colorblind shape backup)

---

## Tier Commitment

**Target Tier: Standard**

**Rationale:** Lanes and Lies is a real-time multiplayer card/strategy game with significant information-density demands — five simultaneous systems during the 10-second PLACEMENT phase, two mana pools, auction price tracking, and hidden objective reasoning. The primary accessibility barriers are visual (color-coded information) and motor (timed inputs). Standard tier addresses both: colorblind modes resolve the color dependency; input timing adjustments address the hard 10-second placement timer.

The WASM browser target means no platform certification requirement (no Xbox XAG, no PS5 guidelines). Standard tier is the appropriate commitment for a solo-dev hackathon-origin project that can realistically implement these features. The art bible (Section 4.6) has already defined colorblind shape-backup for all critical color pairs — implementation effort for colorblind modes is reduced accordingly.

No voiced dialogue exists in the current design; subtitle requirements are minimal. Dropping to Basic would exclude players relying on colorblind modes (estimated 8% of men) and those affected by the 10-second placement timer — an inexcusable exclusion given the low implementation cost.

**In scope beyond tier baseline:**
- Timer extension option for PLACEMENT phase — relevant to the game's specific 10s hard deadline
- Distinct container shapes for current mana vs. reserve mana (not only color) — cognitive safety, not just visual

**Out of scope (documented intentional limitations):**
- Screen reader support for in-game board — Bevy 0.18 has no AccessKit integration; menus only via future effort
- Full subtitle customisation — no voice acting in current design makes this low priority

---

## Visual Accessibility

| Feature | Tier | Status | Notes |
|---|---|---|---|
| Minimum text size — HUD (gold, mana, round number) | Standard | Not Started | 20px minimum for all resource counters at 1080p. Auction price counter is exception: minimum 40px (Section 7.4 typography rule). |
| Minimum text size — card text (cost, ATK, HP, keyword) | Standard | Not Started | Stat badges (ATK diamond, HP gem) minimum 18px. Keyword text floor: 14px. |
| Text contrast — UI on backgrounds | Standard | Not Started | Minimum 4.5:1 for all body text. Auction price counter: 7:1 minimum (time-pressure read). |
| Colorblind mode — Protanopia / Deuteranopia | Standard | **Partially addressed** | Art bible §4.6 defines shape backup: Player A = circle base ring, Player B = diamond base ring. Class icons always shown alongside class colors. Requires formal toggle in settings. Palette shift still needed: shift red combat indicators toward orange-red; verify Sacrier/Cra pair. |
| Colorblind mode — Tritanopia | Standard | Not Started | Shift Ink Blue UI elements toward purple; shift Arcane Gold toward amber. Verify auction escalation track remains readable. |
| Color-as-only-indicator audit | Basic | **Partially addressed** | See table below. Art bible has resolved player-side and class-color pairs; objective dots and auction escalation still need review. |
| UI scaling | Standard | Not Started | Range 75%–150%. HUD scaling independent from menu scaling. |
| Brightness / gamma controls | Basic | Not Started | Exposed in graphics settings. Range: −50% to +50%. |
| Screen flash warning | Basic | Not Started | Pre-launch photosensitivity notice. Audit RESOLUTION combat flash and GAME_OVER objective-destruction burst against Harding FPA standard (max 3 flashes/sec). |
| Motion / animation reduction mode | Standard | Not Started | Reduce: auction panel entrance animation, bid pulse, phase-transition sweep. Cannot eliminate unit movement (board readability). Toggle in settings. |

### Color-as-Only-Indicator Audit

| Location | Color Signal | Non-Color Backup | Status |
|---|---|---|---|
| Player A vs. Player B (unit bases, board edge) | Sky Blue vs. Terracotta | Circle base ring (A) vs. Diamond base ring (B) — art bible §4.6 | **Addressed in art bible** |
| Class identity (Xelor, Sacrier, Iop, Eniripsa, Cra) | Class-specific hues | Class icon always on unit base — art bible §7.5 | **Addressed in art bible** |
| Objective status dots (active vs. destroyed) | Color change | Dot shrinks or cracks on destruction (shape change) | Not Started |
| Auction price escalation track | Blue → Amber → Crimson | Bid number displayed in text; pulse animation reinforces escalation | **Addressed (§4.5)** |
| ATK stat (orange diamond) vs. HP stat (teal gem) | Different hues | Distinct gem shapes (diamond vs. rounded gem) | **Addressed in art bible §3.2** |
| Combat damage numbers (red) vs. healing (gold) | Red vs. Gold | Floating numeral direction (down = damage, up = heal) | Not Started |

---

## Motor Accessibility

| Feature | Tier | Status | Notes |
|---|---|---|---|
| Full input remapping (keyboard + mouse) | Standard | Not Started | Every input rebindable. No two actions bound to same key simultaneously. Persist to browser localStorage or profile. |
| PLACEMENT timer extension | Standard | Not Started | Provide multiplier: 0.5×, 1×, 1.5×, 2×, 3×. At 3× the 10-second placement window becomes 30 seconds. Default: 1×. This is the highest motor-impact feature in the game — the 10s hard deadline is the biggest barrier. |
| Hold-to-press alternatives | Standard | Not Started | Audit all "hold to confirm" inputs. Provide toggle alternative. |
| DRAFT_SHOP ready signal — retractable | Standard | **Addressed in design** | RSM Rule 8: ready signal is retractable at any time until all-ready fires. Prevents accidental early commitment. |
| Auction bid buttons — immediate preset commitments | Standard | **Addressed in design** | Auction bids do not require a separate confirmation step. Misclick mitigation is handled by preset total-commitment labels, 44x44 targets, focus rings, per-button affordability gating, same-frame in-flight disable, one-send semantics, and visible "BIDDING..." feedback. |

---

## Cognitive Accessibility

| Feature | Tier | Status | Notes |
|---|---|---|---|
| Mana pools: distinct container shapes | Standard | Not Started | Current mana = bar shape. Reserve mana = diamond shape. Must be different shapes, not only different colors. Supports players who cannot rely on color alone to distinguish the two pools. |
| PLACEMENT staged disclosure | Standard | Not Started | UI must guide: select card → select lane → select cell → confirm mana split. Should not show mana split input until card and lane are selected. Reduces simultaneous decision count from 4 to 1-at-a-time. |
| Tutorial persistence | Standard | Not Started | All tutorial prompts accessible from pause menu Help section after dismissal. |
| Pause anywhere | Basic | Not Started | Game must be pausable during DRAFT and LOBBY phases. PLACEMENT and RESOLUTION may have "pause requested" indicator that takes effect at next phase boundary — real-time multiplayer constraint. Document server-pause behavior for solo play testing. |
| Phase label always visible | Standard | **Addressed in design** | Art bible §7.1: persistent phase label required at all times (UX flag). Must not rely on animation alone to signal phase changes. |
| Gold counter always visible | Standard | **Addressed in design** | Art bible §7.1: gold counter is never occluded. Always rendered at full HUD-layer opacity. |
| DRAFT_INITIAL: clear objective | Standard | Not Started | At session start, brief overlay confirms: "Select up to 9 cards to keep. You have 45 seconds." Dismissible but retrievable. |

---

## Auditory Accessibility

| Feature | Tier | Status | Notes |
|---|---|---|---|
| Independent volume controls | Basic | Not Started | Music / SFX / UI audio buses, three sliders minimum. Persist to profile. |
| Visual indicators for audio cues | Standard | Not Started | Audit all gameplay-critical SFX. Confirmed visual backups needed for: (1) auction timer final 5s — already has color escalation; (2) PLACEMENT timer countdown — needs visible number, not only tone; (3) RESOLUTION combat outcome — has color (win = gold, loss = blue per §2) + floating damage numbers. |
| No dialogue / voiced content | N/A | N/A | Current design has no voiced dialogue. Subtitle requirements are minimal. |

---

## Per-Feature Accessibility Matrix

| System | Visual | Motor | Cognitive | Auditory | Addressed |
|---|---|---|---|---|---|
| Game Session / Lobby | Class colors on UI | None — unhurried | Class selection is low-pressure | None critical | Partial — class icons backup |
| DRAFT_INITIAL | Card stat readability | None — 45s timer | 9-card selection, moderate load | None | Not Started |
| DRAFT_AUCTION | Auction escalation color track; opponent gold visibility | Immediate preset bid buttons with documented misclick mitigations | Price + own gold + opponent gold + card + hand = 5 elements | Timer final 5s audio cue | Partial — color has text backup; bid motor conflict resolved in UX |
| DRAFT_SHOP | Shop slot readability | None — 30s soft timer, retractable | Moderate — shop + hand + gold | None | Not Started |
| PLACEMENT | Spawn range highlight; opponent side opaque | **10s PLACEMENT TIMER — highest risk** | 4 decisions in 10s | Countdown tone | Partial — staged disclosure planned |
| RESOLUTION | Combat result colors (win/loss) | None — read-only phase | Replay sequence, no decisions | Combat SFX | Partial — color has floating number backup |
| GAME_OVER | Fake objective reveal animation | None | None | Objective destruction burst SFX | Not Started |
| Board / Lane System | Movement arrows; spawn range; unit identity | None | Track 5 lanes simultaneously | None | Partial — art bible shape language |
| Objective System | Objective dots; real/fake visual parity | None | Infer fake vs. real location | Destruction SFX | Partial — dots need shape change on destroy |

---

## Known Intentional Limitations

| Feature | Tier | Why Not Included | Mitigation |
|---|---|---|---|
| Screen reader for in-game board | Comprehensive | Bevy 0.18 has no AccessKit integration for game-world elements | Ensure all critical state accessible via pause menu; plan for future accessibility pass |
| Full subtitle customisation | Comprehensive | No voice acting in current design — low priority | If voice acting is added in future, revisit |
| Mono audio option | Comprehensive | Deferred to post-launch | Low implementation risk — can be added as a patch |
| Tactile/haptic alternatives | Exemplary | No haptic API on WASM/browser | Out of scope for WASM platform |

---

## Open Questions

| Question | Owner | Priority |
|---|---|---|
| Does Bevy 0.18 support accessible UI element names/roles for browser screen reader passthrough? | lead-programmer | Medium |
| What is the minimum PLACEMENT timer extension needed to cover 99th-percentile motor reaction time? (Suggested: 3× = 30s) | ux-designer | High |
| Should PLACEMENT timer pause when the browser tab is backgrounded? (WASM half-open connection risk per network-protocol.md NP-25) | lead-programmer | High |
| Does the staged-disclosure pattern for PLACEMENT require a GDD revision to formally specify? | game-designer | Medium |
