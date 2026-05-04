# Audio Production Queue - 2026-05-04

> Handoff type: Audio asset production queue
> Scope: SFX and short audio cues only. Do not generate audio files in this pass.
> Source manifest: `design/assets/asset-manifest.md`
> Art direction source: `design/art/art-bible.md`

## Source Specs

- `design/assets/specs/board-rendering-assets.md`
- `design/assets/specs/hand-ui-assets.md`
- `design/assets/specs/combat-resolution-assets.md`
- `design/assets/specs/auction-system-assets.md`
- `design/assets/specs/class-system-assets.md`

## Production Rules

- Keep every asset in production queue status; no asset is marked complete here.
- Preserve required filenames from the owning spec when present.
- Proposed paths use lowercase underscore filenames and group by runtime audio area.
- OGG/Vorbis delivery and WAV authoring masters are expected where the owning spec requests both.
- The audio language should support the art bible's phase arc: lobby anticipation, auction pressure, placement dread, and resolution spectacle.

## Priority Bands

- P0 Blocking: needed for current board/objective readability or active gameplay feedback.
- P1 Current Interaction: needed for present UI/combat usability, repeated often enough to require fatigue checks.
- P2 Feature Complete: needed for auction/class/lobby completeness after blocking gameplay cues.
- P3 Advisory/Reused: useful or required by future pass, but not the first implementation blocker.

---

## 1. Board / Objective Blocking SFX

| Asset ID | Asset name | Filename / path | Duration target | Sonic description | Priority | Owning spec file |
|---|---|---|---|---|---|---|
| ASSET-045 | snd_reveal_sting | `assets/audio/board/snd_reveal_sting.ogg` | <= 600ms total | Dry low-mid ink-stamp thud followed immediately by a short staccato minor or minor-seventh pizzicato/plucked-string sting. Tense, brief, not alarming. | P0 Blocking | `design/assets/specs/board-rendering-assets.md` |
| ASSET-046 | snd_unit_advance | `assets/audio/board/snd_unit_advance.ogg` | < 200ms total | Light organic footstep cluster on stone cobble; two to three soft dry impacts, matter-of-fact and stack-safe across five lanes. | P0 Blocking | `design/assets/specs/board-rendering-assets.md` |
| ASSET-047 | snd_objective_destroy_real | `assets/audio/board/snd_objective_destroy_real.ogg` | 1.0-1.5s | Two-phase real objective hit: sharp stone-shattering transient, then full orchestral or choir/brass major bloom. Loudest non-music event. | P0 Blocking | `design/assets/specs/board-rendering-assets.md` |
| ASSET-048 | snd_objective_destroy_fake | `assets/audio/board/snd_objective_destroy_fake.ogg` | < 350ms | Single dry hollow thud with narrow low-mid content, no musical hit, no sustain, no reverb. Anticlimactic contrast to real objective destruction. | P0 Blocking | `design/assets/specs/board-rendering-assets.md` |
| ASSET-050 | snd_objective_attack | `assets/audio/board/snd_objective_attack.ogg` | 350-650ms | Heavy deep resonant thud centered low, like a large wooden siege door absorbing a hit. Physical objective-danger cue without climactic music. | P0 Blocking | `design/assets/specs/board-rendering-assets.md` |
| ASSET-049 | snd_prism_collect | `assets/audio/board/snd_prism_collect.ogg` | 400-600ms including ring tail | Bright crystalline chime in a high register, clean attack with natural ring tail; rewarding and brief without masking board sounds. | P3 Advisory | `design/assets/specs/board-rendering-assets.md` |
| ASSET-051 | snd_trap_trigger | `assets/audio/board/snd_trap_trigger.ogg` | 250-400ms | Two-layer reveal: sharp dry mechanical hit plus card-flip rustle offset shortly after. "Caught you" punctuation, strategic but not threatening. | P3 Advisory | `design/assets/specs/board-rendering-assets.md` |

Production note: ASSET-051 is specified as a single composited file by default. Only split into `snd_trap_trigger_hit.ogg` and `snd_trap_trigger_flip.ogg` if audio authoring requires it and timing is confirmed.

---

## 2. Hand UI Current Interaction SFX

| Asset ID | Asset name | Filename / path | Duration target | Sonic description | Priority | Owning spec file |
|---|---|---|---|---|---|---|
| ASSET-078 | Card Lift SFX | `assets/audio/ui/hand/sfx_card_lift_default.ogg` | 60-80ms | Crisp papery pickup transient with brief air displacement, no reverb and no pitch center. Tactile card stock. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-079 | Valid Targets Appear SFX | `assets/audio/ui/hand/sfx_targets_appear_default.ogg` | ~100ms | Subtle upper-register crystalline shimmer, nearly subliminal, triggered once per valid drag gesture. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-080 | Successful Stage SFX | `assets/audio/ui/hand/sfx_card_stage_default.ogg` | 80-120ms | Warm weighted thunk or placement click, like a wooden game piece on stone. Heavier than lift; no reverb. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-081 | Snap-Back / Invalid Drop SFX | `assets/audio/ui/hand/sfx_card_snapback_default.ogg` | ~80ms | Soft quick whoosh-back, low-mid and quiet, communicating physical reversal without error/buzzer language. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-082 | Instant Card Staged SFX | `assets/audio/ui/hand/sfx_card_instant_stage_default.ogg` | ~100ms | ASSET-080-style weighted thunk plus a brief cool crystalline overtone after the impact for magical confirmation. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-083 | Submit SFX | `assets/audio/ui/hand/sfx_submit_default.ogg` | ~400ms | Sharp leading click plus resonant natural ring that implies finality; must remain audible at low browser volume. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-084 | Timer Urgency SFX | `assets/audio/ui/hand/sfx_timer_urgency_default.ogg` | ~300-500ms | Single heartbeat-register cue at the five-second mark. No loop and no tick sequence. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-085 | Card Acquired SFX | `assets/audio/ui/hand/sfx_card_acquired_default.ogg` | ~150ms | Light ascending two-note metallic chime, pleasant under repeated DRAFT_INITIAL acquisition playback. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-086 | Hand Full SFX | `assets/audio/ui/hand/sfx_hand_full_default.ogg` | ~200ms | Soft neutral mid-frequency bell, single strike, informative rather than scolding. Lower and softer than card acquired. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |
| ASSET-087 | Reserve Adjust Click SFX | `assets/audio/ui/hand/sfx_reserve_adjust_default.ogg` | ~50ms | Soft mid-register button click with no reverb, low dynamic volume, designed not to fatigue during rapid repetition. | P1 Current Interaction | `design/assets/specs/hand-ui-assets.md` |

Production note: All hand UI sounds belong to the `ui_hand` channel, mono, 44100 Hz per the owning spec. ASSET-084 must be authored for one-shot playback only.

---

## 3. Combat SS3 / SS4 / SS6 SFX

| Asset ID | Asset name | Filename / path | Duration target | Sonic description | Priority | Owning spec file |
|---|---|---|---|---|---|---|
| ASSET-156 | FIRST STRIKE Impact SFX | `assets/audio/combat/sfx_combat_first_strike_hit.ogg` | 120-180ms target, synced inside 200-250ms attack beat | Brighter, sharper, and more immediate than standard combat impact. Should read as timing advantage and align with Prism White impact flashes. | P0 Blocking - SS3 | `design/assets/specs/combat-resolution-assets.md` |
| ASSET-158 | Unit Death SFX | `assets/audio/combat/sfx_combat_unit_death.ogg` | 250-350ms target, matching 350ms death animation ceiling | Short final unit-collapse cue with crimson physicality, not cinematic. Should leave room for DEATH trigger gold pulse if present. | P0 Blocking - SS4 | `design/assets/specs/combat-resolution-assets.md` |
| ASSET-157 | Standard Combat Impact SFX | `assets/audio/combat/sfx_combat_standard_hit.ogg` | 120-200ms target, synced inside 200-250ms attack beat | Warm, physical, mid-register impact. Less sharp than FIRST STRIKE and suitable for repeated simultaneous lane hits. | P0 Blocking - SS6 | `design/assets/specs/combat-resolution-assets.md` |
| ASSET-159 | SHIELD Absorb SFX | `assets/audio/combat/sfx_combat_shield_absorb.ogg` | 100-180ms target | Blocked-force cue with a firm defensive body, no damage-number implication. Must communicate "hit was absorbed." | P0 Blocking - SS3/SS6 | `design/assets/specs/combat-resolution-assets.md` |
| ASSET-160 | SHIELD Break SFX | `assets/audio/combat/sfx_combat_shield_break.ogg` | 180-250ms target | Distinct consumed-state cue, more brittle or dispersive than absorb, timed to the Prism White burst. | P0 Blocking - SS3/SS6 | `design/assets/specs/combat-resolution-assets.md` |
| ASSET-162 | COUNTERATTACK Response SFX | `assets/audio/combat/sfx_combat_counterattack.ogg` | 120-200ms target | Reactive snap after incoming damage, not a second generic hit. Should imply retaliation and stay distinct from standard impact. | P1 Current Combat | `design/assets/specs/combat-resolution-assets.md` |
| ASSET-161 | Kill Gold Reward SFX | `assets/audio/combat/sfx_combat_kill_gold_reward.ogg` | 150-250ms target | Small Arcane Gold reward cue for +1 kill gold, positive but restrained so it does not compete with objective destruction. | P1 Current Combat | `design/assets/specs/combat-resolution-assets.md` |
| ASSET-155 | Placement Reveal Flip SFX | `assets/audio/combat/sfx_combat_reveal_flip.ogg` | 80-100ms target | Five-lane simultaneous flip cue; fast card-back reveal with Prism White edge-flash feel. Included here as combat-owned reveal prerequisite. | P1 Reveal Prerequisite | `design/assets/specs/combat-resolution-assets.md` |

Production note: The combat asset spec states audio direction is pending. Durations above are production targets derived from the combat GDD animation timing and the owning asset names; final sound-designer timing may refine them without changing asset IDs.

---

## 4. Auction SFX

| Asset ID | Asset name | Filename / path | Duration target | Sonic description | Priority | Owning spec file |
|---|---|---|---|---|---|---|
| ASSET-175 | Auction Ambient Urgency Tone Loop | `assets/audio/auction/audio_auction_urgency_loop.ogg` | 6-10s seamless loop target, code-faded on exit | Cool pressure bed that starts on DRAFT_AUCTION entry and separates auction pressure from DRAFT_SHOP calm. Must not mask bid/timer cues. | P2 Feature Complete | `design/assets/specs/auction-system-assets.md` |
| ASSET-176 | Accepted Bid Ascending SFX | `assets/audio/auction/audio_auction_bid_accepted.ogg` | 80-140ms target | Short ascending bid cue. Rapid bids should form an escalating pitch series without becoming musical clutter. | P2 Feature Complete | `design/assets/specs/auction-system-assets.md` |
| ASSET-177 | Auction Red-Zone Countdown Tick Cue | `assets/audio/auction/audio_countdown_tick_loop.ogg` | Reuse ASSET-021 tick; one-second red-zone cadence | Audible countdown tick below five seconds. This is an auction-owned trigger using the shared countdown tick audio file. | P3 Reuse | `design/assets/specs/auction-system-assets.md` |
| ASSET-178 | Timer Reset Reverse-Tick SFX | `assets/audio/auction/audio_auction_timer_extend.ogg` | 120-220ms target | Brief reverse-tick or inhaling clock gesture for timer extension after an accepted bid. | P2 Feature Complete | `design/assets/specs/auction-system-assets.md` |
| ASSET-179 | Auction Won By Self Sting | `assets/audio/auction/audio_auction_won_self.ogg` | 400-700ms target | Clear self-win settlement sting. Satisfying but not a match victory fanfare. | P2 Feature Complete | `design/assets/specs/auction-system-assets.md` |
| ASSET-180 | Auction Won By Opponent Sting | `assets/audio/auction/audio_auction_won_opponent.ogg` | 350-600ms target | Neutral resolved sting for opponent win, distinct from self-win without reading as defeat. | P2 Feature Complete | `design/assets/specs/auction-system-assets.md` |
| ASSET-181 | No-Bid Card Gone SFX | `assets/audio/auction/audio_auction_no_bid_card_gone.ogg` | 250-450ms target | Muted card-gone cue in a minor register. Communicates lost opportunity without dramatic penalty. | P2 Feature Complete | `design/assets/specs/auction-system-assets.md` |

Production note: Auction cues should reinforce pressure and settlement, not match outcome. Self-win, opponent-win, and no-bid must be clearly distinguishable.

---

## 5. Class / Lobby SFX

| Asset ID | Asset name | Filename / path | Duration target | Sonic description | Priority | Owning spec file |
|---|---|---|---|---|---|---|
| ASSET-123 | Class Select Hover SFX | `assets/audio/ui/audio_class_select_hover.ogg` | 80-120ms | Short exploratory UI hover: warm hollow woodblock or light mallet transient with brief small-room tail. | P2 Feature Complete | `design/assets/specs/class-system-assets.md` |
| ASSET-124 | Class Confirm / Ready SFX | `assets/audio/ui/audio_class_confirm_ready.ogg` | 200-280ms | Resonant wooden/bone clack followed by ascending two-note warm metallic pentatonic chime. Decisive class commitment. | P2 Feature Complete | `design/assets/specs/class-system-assets.md` |
| ASSET-125 | Opponent Class Reveal SFX | `assets/audio/ui/audio_class_opponent_reveal.ogg` | 350-500ms | Low drum onset plus rising three-note harp-like arpeggio and optional short tail. Neutral dramatic reveal, not celebratory. | P2 Feature Complete | `design/assets/specs/class-system-assets.md` |
| ASSET-126 | Reserve Gain SFX | `assets/audio/ui/audio_reserve_gain.ogg` | 180-240ms | Cool high crystalline reserve-diamond chime plus brief low sub-bass pulse. Generic reserve deposit cue. | P2 Feature Complete | `design/assets/specs/class-system-assets.md` |
| ASSET-127 | Ready Retract SFX | `assets/audio/ui/audio_class_ready_retract.ogg` | 120-180ms | Descending two-note counterpart to confirm, slightly muffled with soft paper-shuffle texture. Valid retraction, not error. | P2 Feature Complete | `design/assets/specs/class-system-assets.md` |

Production note: Class/lobby cues are specified as OGG Vorbis, 96 kbps mono in `assets/audio/ui/`. ASSET-123 needs hover debouncing; ASSET-126 may need short staggered playback for concurrent reserve triggers.

---

## Blocker Status

- No blocker to starting audio asset production from this queue.
- Technical caution: `hand-ui-assets.md` and `class-system-assets.md` both flag Bevy 0.18 audio API verification before integration; this does not block asset authoring.
- Design caution: ASSET-051 split-file fallback and combat/auction inferred duration targets should be confirmed by audio direction before final mastering.

## Changed Asset Statuses

None. This handoff does not change manifest status, approval status, implementation status, or asset completion state.
