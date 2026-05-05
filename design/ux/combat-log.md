# UX Spec: Combat Log / Resolution Event Replay UI

> **Status**: In Design
> **Author**: user + ux-designer
> **Last Updated**: 2026-05-05
> **Journey Phase(s)**: RESOLUTION
> **Template**: UX Spec
> **Input Methods**: Mouse + Keyboard. No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard (`design/accessibility-requirements.md`)
> **Source Docs**: `design/gdd/combat-resolution.md`, `production/epics/combat-resolution/story-011-resolution-event-log.md`, `production/epics/board-rendering/story-006-resolution-anim-queue-and-phase-buffering.md`, `design/ux/hud.md`

---

## Purpose & Player Need

The Combat Log / Resolution Event Replay UI makes the server-authored `S2CResolutionEvent` readable to players while the board animation is playing. Its job is not to replace board playback. Its job is to give players a compact, ordered explanation of what just happened so RESOLUTION remains active reading instead of passive watching.

The player arrives at this UI wanting to answer:

- What happened first, second, and third?
- Why did this unit die, survive, move, trigger, or award gold?
- Which lane or objective should I care about next round?
- Did the replay preserve simultaneous-resolution rules instead of implying a false lane-by-lane sequence?

This spec defines the UX target that Board Rendering can implement once COMBAT-011 stabilizes the complete `S2CResolutionEvent` schema and ordering.

---

## Player Context on Arrival

The UI appears during `RESOLUTION_EXECUTING`, after `S2CPlacementReveal` has revealed both players' placements and before the client resumes DRAFT UI. Player input is locked for gameplay actions; the player's task is observation, interpretation, and planning.

The assumed emotional state is high-attention tactical reading. The player is not choosing an action, but they are looking for information that changes the next round: lane pressure, objective damage, keyword reveals, unit survival, and gold swings.

The UI must preserve the existing HUD rule that RESOLUTION dims persistent HUD to 70% and makes the board the hero. The log is secondary evidence, not the focal layer.

---

## Navigation Position

This is not a standalone screen. It is a contextual RESOLUTION overlay attached to the board presentation.

Navigation position:

`Game Session -> Board -> RESOLUTION playback -> Combat Log / Resolution Event Replay UI`

The log has no main-menu entry and no persistent navigation destination in M2. It is automatically available only while a resolution script exists for the current round.

---

## Entry & Exit Points

| Entry Source | Trigger | Player carries this context |
|---|---|---|
| `S2CPlacementReveal` | Board enters reveal state and starts placement reveal playback | Both players' committed placements become visible simultaneously |
| `S2CResolutionEvent` | Complete ordered batch arrives on reliable channel | Full replay event list, grouped by sub-step for playback and log display |
| Buffered resolution script | `S2CResolutionEvent` arrives before placement reveal finishes | Log waits in "queued" state until placement reveal is complete |

| Exit Destination | Trigger | Notes |
|---|---|---|
| DRAFT_SHOP board state | Animation queue drains and any buffered `S2CPhaseChanged(DRAFT_SHOP)` is applied | The log collapses with RESOLUTION UI; no player input is restored before playback completes |
| Desync recovery state | Invalid or out-of-range sub-step is detected | Show concise recovery row, request snapshot through Board Rendering recovery path |
| Reconnect snapshot state | `S2CGameSnapshot` rebuilds board | Prior replay log is discarded unless future replay history is explicitly designed |

---

## Layout Specification

### Information Hierarchy

1. Current sub-step label and progress through the six-step resolution sequence.
2. Board animation in the center of the screen.
3. Current and just-completed log entries for the active sub-step.
4. Lane, source, target, amount, and result for each entry.
5. Gold, objective, and keyword entries that explain strategic consequences.
6. Replay controls, only if playback control is enabled.
7. Detailed historical rows for earlier sub-steps.

The log must make event ordering legible without implying that global simultaneous passes resolve lane-by-lane. The UI shows each sub-step as a group, with same-step rows ordered chronologically by server `trigger_index` or emission order.

### Layout Zones

The layout uses a slim right-side log rail plus a top-center sub-step strip. This keeps the board center unobstructed and aligns with the HUD perimeter-ring philosophy.

| Zone | Position | Contents | Notes |
|---|---|---|---|
| Sub-step strip | Top-center, below phase label/objective dots | Six numbered step markers, current step label, compact progress | Persistent during RESOLUTION; never animation-only |
| Active log rail | Right edge, inside safe margin | Current sub-step event rows, newest relevant entries visible | Width capped so it does not cover board cells on common 16:9 layouts |
| Optional controls | Bottom-right above mana cluster, only when enabled | Pause/resume, step back, step forward, replay speed | Hidden in M2 unless implementation accepts replay controls |
| Recovery row | Top of active log rail | Invalid script, waiting for reveal, snapshot requested | Text-based; never modal |

### Component Inventory

| Component | Type | Content | Interactive? | Pattern |
|---|---|---|---|---|
| Sub-step strip | Data display | Steps 1-6: Placements, Charge X, First Strike, Remove Dead, Movement, Combat/Objectives | No | New pattern: Resolution Stepper |
| Log rail container | Read-only list | Event rows grouped under current sub-step heading | Scroll only if event count exceeds visible space | New pattern: Resolution Event Rail |
| Event row | Data display | Icon/glyph, lane, actor, target, value, result | No in default mode | New pattern: Event Row |
| Damage entry | Data display | `Lane N`, source unit, target unit/objective, damage amount, shield/armor note | No | Event Row |
| Gold entry | Data display | `+1 Kill Gold`, `+3 Objective Gold`, player side, origin | No | PTN-FDB-003 for numeric delta; Event Row |
| Objective entry | Data display | Lane, HP after, destroyed state, fake/real reveal if known | No | PTN-DSP-006 status dot language |
| Keyword entry | Data display | Keyword name, source unit, target/result | No | Event Row |
| Replay controls | Icon buttons | Pause/resume, previous step, next step, speed | Optional | New pattern: Replay Controls |
| Reduced-motion indicator | Text/icon state | `Reduced motion` compact label only in settings/debug surfaces, not in live log | No | Existing accessibility setting |

### ASCII Wireframe

```text
+--------------------------------------------------------------------------+
| [Own Fig]     [Objective Dots] [RESOLUTION - Round N]     [Opp Fig]       |
|                         [1][2][3][4][5][6] First Strike                  |
+--------------------------------------------------------------------------+
|                                                                          |
|  5 lane board remains primary focus                                      |
|                                                                          |
|                                                     +------------------+ |
|                                                     | FIRST STRIKE     | |
|                                                     | L2 Unit A -> B 3 | |
|                                                     | L4 Shield blocks | |
|                                                     | L4 Counterattack | |
|                                                     |                  | |
|                                                     | Previous:        | |
|                                                     | Placement x4     | |
|                                                     +------------------+ |
|                                                                          |
+--------------------------------------------------------------------------+
| Own Gold   Opp Gold                                  Mana / Reserve       |
| Hand cards dimmed during RESOLUTION                                      |
+--------------------------------------------------------------------------+
```

---

## States & Variants

| State / Variant | Trigger | What Changes |
|---|---|---|
| Waiting for reveal | `S2CResolutionEvent` arrives before `S2CPlacementReveal` finishes | Rail shows "Replay queued"; no event rows animate yet |
| Default playback | Valid `S2CResolutionEvent` with events | Stepper advances by sub-step; rail shows current group |
| Dense round | Current sub-step has more rows than rail can show | Show top priority rows plus `+N more`; allow scroll after playback completes if replay controls/history are enabled |
| Empty sub-step | A sub-step executes but produces no visible gameplay entries beyond `SubStepBegin` | Stepper still advances; rail row says "No board changes" |
| Shield-blocked damage | `CombatDamage` amount is 0 and shield flag is true | Damage row uses shield icon/glyph and "blocked", not a red damage number |
| Objective destroyed | `ObjectiveDestroyed` entry appears | Objective row is promoted above lower-priority entries for that sub-step |
| Gold awarded | `GoldAwarded` entry appears | Gold row appears near corresponding kill/objective row and mirrors contextual board popup |
| Invalid sub-step | Event contains sub-step outside 1..=6 | Stop playback, show recovery row, Board Rendering requests snapshot |
| Reconnect/snapshot | Snapshot rebuild occurs during or after RESOLUTION | Current replay rail clears; board state becomes authoritative |
| Reduced motion | Player enables motion reduction | Stepper and rail use instant state changes; board movement remains because it is required for comprehension |

---

## Interaction Map

Default RESOLUTION is read-only. Hand cards, shop slots, placement controls, and modal UI are suppressed per Combat Resolution R7. The log rail does not capture gameplay input.

| Component | Action | Input | Immediate Feedback | Outcome |
|---|---|---|---|---|
| Log rail | Passive read | None | Active sub-step row is visually emphasized | Player reads chronological explanation |
| Log rail overflow | Scroll after playback completes | Mouse wheel / scrollbar | Rail scrolls within capped area | Player reviews hidden rows without blocking phase transition in M2 only if history persists |
| Pause/resume replay | Optional, not M2 default | Mouse click / Space if focused | Icon toggles; playback timer pauses | `AnimQueue` timer pauses locally; network state remains authoritative |
| Step forward/back | Optional, not M2 default | Mouse click / arrow keys if focused | Stepper moves to adjacent sub-step | Replay view seeks within local script only |
| Speed control | Optional, not M2 default | Mouse click | Label cycles `1x`, `1.5x`, `2x` | Local playback speed changes; disabled under reduced-motion if it harms readability |

Replay controls are optional because the Board Rendering GDD currently accepts the watch-the-tape trade-off and treats "long press to fast-forward" as out of scope for M2. If controls are not implemented, the UI still needs passive log visibility.

---

## Events Fired

The Combat Log UI is display-only and must not create gameplay events. It consumes existing server and presentation events.

| Player Action / UI Change | Event Fired | Payload / Data |
|---|---|---|
| `S2CResolutionEvent` intake | No new gameplay event | Consumes server batch |
| Rail displays event row | Optional analytics only | `resolution_log_row_viewed`, round, sub_step, row_type, if analytics exists |
| Replay pause | Optional presentation event | `ReplayPlaybackPaused { round, sub_step }`; local only |
| Replay resume | Optional presentation event | `ReplayPlaybackResumed { round, sub_step }`; local only |
| Replay seek | Optional presentation event | `ReplayPlaybackSeek { round, from_sub_step, to_sub_step }`; local only |
| Invalid sub-step recovery | Existing Board Rendering recovery path | `C2SRequestSnapshot` is owned by Board Rendering, not this UX spec |

Persistent game state must never be modified by this UI.

---

## Transitions & Animations

The log appears with RESOLUTION playback after placement reveal begins. It fades in at a fixed right-edge position over 80ms; it never slides from off-screen. Each sub-step transition updates the stepper and rail heading with an 80ms crossfade.

Event rows enter in chronological order within the current sub-step using a short opacity fade and no positional travel. Multiple same-target damage entries stagger by a few frames only when required for readability, matching Combat Resolution R4.

Reduced-motion behavior:

- Remove row fade/stagger; rows appear instantly.
- Remove stepper pulse; current step changes by static highlight and text label.
- Preserve unit movement on the board because movement is gameplay information, not decoration.
- Preserve damage/gold/objective text updates; remove scale pulses if they are nonessential.
- Avoid repeated flashes. RESOLUTION combat flash and objective destruction must be audited against the accessibility photosensitivity note.

---

## Data Requirements

| Data | Source System | Read / Write | Notes |
|---|---|---|---|
| Ordered event list | Combat Resolution / COMBAT-011 | Read | Single `S2CResolutionEvent.events` batch in chronological `(sub_step, trigger_index)` order |
| Sub-step entries | Combat Resolution | Read | Exactly one `SubStepBegin` per executed sub-step |
| Damage entries | Combat Resolution | Read | Include all `CombatDamage`, including non-lethal and shield-blocked hits |
| Unit removal entries | Combat Resolution | Read | Needed for death rows and kill attribution |
| Gold entries | Combat Resolution / Economy reward path | Read | `GoldAwarded` rows for kill gold and objective gold; no standalone mid-resolution gold update required |
| Objective entries | Combat Resolution / Objective System | Read | `ObjectiveDamage`, `ObjectiveDestroyed`, HP after, owner/lane, fake flag when visible |
| Keyword entries | Combat Resolution / Keyword System | Read | APPEARANCE, DEATH, COUNTERATTACK, FINAL_BLOW, and other emitted keyword activations |
| Movement entries | Combat Resolution / Board Rendering | Read | `UnitMoved`, lane/cell deltas, sub-step |
| Phase changes | Round State Machine | Read | `S2CPhaseChanged(DRAFT_SHOP)` may be buffered until queue drains |
| Replay playback state | Board Rendering | Read | `AnimQueue`, active group, queue drained state |
| Accessibility settings | Settings / Accessibility | Read | Reduced motion, UI scale, colorblind mode |

The UI must tolerate missing optional fields by falling back to terse but honest rows, e.g. "Lane 3 damage event" rather than inventing actor names.

---

## Event Ordering Presentation

`S2CResolutionEvent` is presented as six global passes:

1. Apply Placements
2. Charge X Movement
3. First Strike
4. Remove Dead
5. Movement
6. Combat and Objectives

Rules:

- The stepper advances only after the previous visual group completes.
- Rows are grouped by `sub_step`.
- Within a sub-step, rows preserve server emission order.
- Same-sub-step rows may be visually clustered by lane only if the row order number remains visible and the grouping does not reorder events.
- The UI must not imply that lane 1 fully resolves before lane 5 for the whole round. Lane order is only a tiebreaker inside specific sub-step rules.
- `S2CPhaseChanged(DRAFT_SHOP)` must not visually resume DRAFT UI before the full replay has displayed.

---

## Entry Types

### Damage Entries

Damage rows must show lane, source, target, amount, and special handling.

Examples:

- `L2 - Bow Meow -> Gobball: 3 damage`
- `L4 - Shield blocked Iop Strike`
- `L3 - Counterattack -> Sacrier: 1 damage`

Damage color is supplementary. Damage direction, icon/glyph, and numeric text carry the meaning.

### Gold Entries

Gold rows must explain both amount and reason:

- `+1 Kill Gold - Player A - L2 Gobball removed`
- `+3 Objective Gold - Player B - L5 objective destroyed`

Gold entries should appear near the corresponding kill/objective row and should match the HUD gold counter delta, but they must not require the player to look away from the board to understand the reward origin.

### Objective Entries

Objective rows must show lane, damage, HP after, and destruction when applicable.

Examples:

- `L5 Objective: 2 damage, HP 1`
- `L1 Objective destroyed: +3 gold`
- `L3 Fake objective revealed`

Real/fake presentation must obey the objective identity visibility rules already established by the objective and board systems.

### Keyword Entries

Keyword rows explain why normal expectations changed:

- `APPEARANCE triggered`
- `DEATH triggered`
- `COUNTERATTACK triggered`
- `FINAL BLOW triggered`
- `SHIELD consumed`
- `STUN suppressed movement`

Keyword names are text labels, not icons alone. If a keyword icon is used, the row still includes the keyword text.

---

## Accessibility

Standard tier applies.

| Requirement | UX decision |
|---|---|
| Text contrast | Rail text uses Ivory or Prism White on Ink Blue/Void background at 4.5:1 minimum |
| Minimum text size | Event rows use 16px minimum at 1080p; sub-step label 20px minimum; numeric damage/gold values 20px minimum |
| Text density | Rail displays 4-6 active rows comfortably; dense rounds collapse lower-priority rows behind `+N more` rather than shrinking text |
| Color independence | Damage, gold, shield, objective, and keyword rows include text plus icon/glyph; color is never the only signal |
| Motion reduction | See reduced-motion behavior in Transitions & Animations |
| Photosensitivity | Combat flash and objective destruction effects must be audited; log rows must not flash repeatedly |
| Keyboard | If replay controls exist, all controls are reachable by Tab with visible focus states and 44x44 CSS px targets |
| Screen reader | In-game board screen reader support is out of current scope; text rows should still be structured so future accessibility work can expose them |
| UI scaling | Rail must remain usable at 75%-150% UI scale and browser zoom without overlapping board-critical content |

The log must not solve dense combat by shrinking text below readable floors. When space is constrained, summarize.

---

## Localization Considerations

Event row copy must be short and tokenized. Avoid sentence-shaped rows where possible.

Preferred row structure:

`[Lane] [Actor] -> [Target]: [Value] [Result]`

Text expansion rules:

- Lane labels should support compact localization (`L2`, `Lane 2`, translated equivalent).
- Keyword names may expand by 40%; rows must allow wrapping to two lines.
- Numeric values and icons must remain aligned when text wraps.
- Replay controls should use icons plus accessible names, not long visible labels.
- Do not bake English keyword names into rendered textures.

---

## Acceptance Criteria

- [ ] The log rail appears during RESOLUTION without covering the central board lanes or objective endpoints at the target 16:9 desktop layout.
- [ ] The sub-step strip always shows which of the six resolution passes is currently playing using persistent text, not animation alone.
- [ ] `S2CResolutionEvent.events` rows are grouped by sub-step and displayed in server chronological order without client-side reordering.
- [ ] Damage rows show lane, source, target, and amount; shield-blocked damage is represented as a blocked event rather than omitted.
- [ ] Gold rows show amount, player, and reason for both +1 kill gold and +3 objective gold.
- [ ] Objective rows show lane, HP-after, destruction state, and fake/real reveal only when that identity is visible to the player.
- [ ] Keyword rows include readable keyword text for APPEARANCE, DEATH, COUNTERATTACK, FINAL_BLOW, SHIELD, and STUN-related events.
- [ ] `S2CPhaseChanged(DRAFT_SHOP)` does not visually restore DRAFT UI until the replay queue has drained.
- [ ] Out-of-range sub-step values stop playback and show a text recovery row while Board Rendering requests a snapshot.
- [ ] Reduced-motion mode removes log row motion and stepper pulse while preserving board movement necessary for comprehension.
- [ ] Event row text remains at or above 16px at 1080p and does not shrink below readability floors in dense rounds.
- [ ] If replay controls are implemented, every control has a visible focus state, 44x44 CSS px target, and no persistent gameplay state writes.

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-CL-1 | Are replay controls in M2, or should the first implementation be passive log-only with no pause/seek/speed controls? | product / board-rendering | High |
| OQ-CL-2 | What is the maximum visible event row count before summarization: 4, 5, or 6 rows? | ux-designer | Medium |
| OQ-CL-3 | Should the log persist for post-round review during DRAFT_SHOP, or disappear immediately when RESOLUTION playback drains? | product / ux-designer | Medium |
| OQ-CL-4 | Should row priority in dense rounds be fixed as objective > gold > death/removal > damage > movement > keyword, or should keyword triggers that cause state changes rank above damage? | game-designer / ux-designer | Medium |
| OQ-CL-5 | Should analytics events be emitted for replay pause/seek once controls exist, or is local-only presentation sufficient? | analytics / producer | Low |

---

## Blockers

- COMBAT-011 must stabilize the complete `S2CResolutionEvent` schema and ensure the batch contains the required CR-32 event categories.
- Board Rendering Story 006 remains blocked until final `ResolutionEvent` variants and ordering are available for production dispatch.
- Replay controls cannot be finalized until product decides whether M2 supports pause/seek/speed or passive log-only playback.

---

## Downstream Stories / Assets Unlocked

- `production/epics/board-rendering/story-006-resolution-anim-queue-and-phase-buffering.md`: can map `S2CResolutionEvent` to UX-visible sub-step groups and buffered phase behavior.
- COMBAT-011 implementation can validate that every required CR-32 event category has a player-facing row target.
- Card Animations and Board Rendering can coordinate damage, gold, objective, and keyword row timing with board VFX.
- Interaction pattern library follow-up: add `Resolution Stepper`, `Resolution Event Rail`, `Event Row`, and optional `Replay Controls`.
- UI asset follow-up: compact glyphs/icons for damage, shield blocked, gold, objective damage/destroyed, keyword trigger, and recovery state.
