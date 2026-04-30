# Review Log — Card Animations

## Review — 2026-04-30 (Pass 5) — Verdict: APPROVED
Scope signal: L
Specialists: lean (single-session, no specialist agents)
Blocking items: 0 | Recommended: 4
Summary: Pass 5 found no new blocking items. The GDD is implementation-ready. One inline fix applied: V.3 audio section stale "1.5 s overlay hold" corrected to "400 ms settlement overlay" (OQ-CA-07 was resolved in Pass 4 but the text was not updated). Three cross-doc verifications deferred to pre-implementation sprint check: board-rendering.md F4 concurrent stagger formula, game-config.md sub-step minimum floor (451 ms), and shop-auction-ui.md settlement overlay duration sync. All 14 remaining OQs are Bevy 0.18 API verification gates correctly marked as pre-implementation blockers, not design gaps. Architecture (domain-event indirection, AnimGroup/AnimQueue, Decoration Test) confirmed sound after 4 prior revision rounds. PRE-LAUNCH BLOCKING: GAME_OVER 599 ms mitigation documented but intentionally deferred per friend-game scope.
Prior verdict resolved: Yes — Pass 4 (2026-04-30) NEEDS REVISION resolved in-session


## Review — 2026-04-30 (Pass 4) — Verdict: NEEDS REVISION → resolved in-session
Scope signal: L
Specialists: game-designer · systems-designer · qa-lead · gameplay-programmer · audio-director · ux-designer · network-programmer · creative-director (senior synthesis)
Blocking items: 11 | Significant: 9 | Advisory: 14+
Summary: Pass 4 surfaced 11 blockers across 7 specialist domains. The settlement overlay's D1 reframing (accepted in Pass 3) was rejected as circular reasoning: mental-state transition is not information delivery, and the overlay fails D2. Cut to 400ms. Three critical architectural issues discovered: F2 Tuning Knobs safe-range column was wrong at sub-step minimum (context-dependent, not a fixed 549ms ceiling); F1+GAME_OVER stagger sequence had an unhandled interleaving (Board Rendering now polls StagedObjectiveRevealQueue.is_empty() before GAME_OVER transition); and Rule C-13's hover invariant contradicted CA-24's 2-entity Playing assertion (restated to "scaling toward 1.12× only"). CA-5a was vacuous (1-group queue, 0 always equals 0); CA-5c was untestable against MessageReader (messages don't persist across update() calls); OQ-CA-13 settlement audio promoted to BLOCKING. All 11 blockers resolved in-session. 5 significant items also fixed (ResolutionObjectiveReveal build spec, StagedObjectiveRevealQueue u8 type, F3 stub, BackgroundColor OQ-CA-10 extension, CA-14 reclassification). GAME_OVER 599ms compromise tagged PRE-LAUNCH BLOCKING.
Prior verdict resolved: Yes — Pass 3 (2026-04-30) NEEDS REVISION resolved

## Review — 2026-04-30 (Pass 3) — Verdict: NEEDS REVISION → resolved in-session
Scope signal: L
Specialists: game-designer · systems-designer · qa-lead · gameplay-programmer · audio-director · ux-designer · network-programmer · creative-director (senior synthesis)
Blocking items: 15 | Recommended: 15
Summary: Pass 3 found 15 new blockers despite 37 resolved in prior passes. Architecture remains sound. Critical discoveries: ghost events (ResolutionGroupReady/FogLiftReady had no upstream emitters), DamageNumberSpawnRequested missing from domain event table, CA-21 assertion fundamentally wrong (BoardCell is a PlayTarget enum variant, not an ECS component), CA-24 contradicted Rule C-13, and 5 Card Animations config fields were absent from game-config.md (startup panic). Also: F2 variable range/assert ambiguity at 550ms boundary, V.1 duration hierarchy numerically inverted, GAME_OVER drain ownership undefined. All 15 blockers resolved in-session. GroupDrainedSignal introduced as sole CA emission (GAME_OVER path). User kept GAME_OVER 599ms compromise for friend-game scope. 14 OQs remain (5 Bevy API verification blocks all implementation).
Prior verdict resolved: Yes — Pass 2 (2026-04-30) NEEDS REVISION resolved

## Review — 2026-04-30 (Pass 2) — Verdict: NEEDS REVISION
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, gameplay-programmer, performance-analyst, audio-director, ux-designer, network-programmer, creative-director (senior synthesis)
Blocking items: 24 new | Recommended: 15
Summary: Re-review after Pass 1's 13 blockers were resolved in-session. Pass 2 surfaced 24 new blockers across 6 specialist domains. The architecture (domain-event indirection, AnimGroup/AnimQueue, Decoration Test philosophy) remains sound. Critical gaps: OQ-CA-06 was mislabeled "Resolved" while pointing to the wrong Bevy 0.18 API (`add_event` does not exist in 0.17+; correct: `add_message`/`MessageReader`); `PlacementCancelAllAnimsRequested` was missing from the domain event table (CA-21 had no delivery mechanism); F2 tuning table had three mathematical errors; `impact_flash_audio_offset_ms` was a phantom config key with a wrong derivation (67 ms at 60 fps; corrected to ~17 ms). All 24 blockers resolved in-session. 12 OQs remain open (5 Bevy API verification, 3 cross-system recommendations, 4 new). Re-review in fresh session recommended.
Prior verdict resolved: Yes — Pass 1 (2026-04-30) NEEDS REVISION resolved

## Review — 2026-04-30 — Verdict: NEEDS REVISION
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, gameplay-programmer, network-programmer, performance-analyst, ux-designer, audio-director, creative-director (senior synthesis)
Blocking items: 13 | Recommended: 10
Summary: The GDD's core architecture (domain-event indirection, restrained animation philosophy, decoration test) is conceptually sound. Critical issues were: `add_message` is not Bevy 0.18 API (should be `add_event`); `sprite.color.set_alpha()` unverified (OQ-CA-10 added); DRAFT_INITIAL violated Rule C-14 (fixed via card-draw sequencing at t+350ms); audio timing contract had no delivery mechanism (fixed via offset-based model); PLACEMENT→RESOLUTION normal-path transition was unspecified (force-cancel edge case + CA-21 added); F1 formula had wrong i-range (0–4→0–3); F2 constraint allowed silent violation at max tuning values. All 13 blockers resolved in-session. 10 recommended items addressed. Re-review in fresh session recommended.
Prior verdict resolved: No — first review
