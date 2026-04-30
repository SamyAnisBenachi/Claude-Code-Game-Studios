# Board Rendering — Review Log

## Review — 2026-04-30 (R4) — Verdict: NEEDS REVISION → resolved in-session
Scope signal: XL
Specialists: game-designer, systems-designer, network-programmer, qa-lead, performance-analyst, gameplay-programmer (Bevy 0.18), creative-director (senior synthesis)
Blocking items: 7 | Recommended: 5+
Summary: R4 trigger was OQ-BR-06 resolution in network-protocol.md. Six new blocker categories surfaced: (1) compile-time API error — `sprite.color.set_alpha()` does not exist in Bevy 0.18; (2) OQ-BR-06 stale text at 3 locations, 3 ACs still gated; (3) single-shot snapshot recovery lacks rate-limit contract (NP-43 server cooldown can silently drop requests, leaving client frozen up to 30s); (4) draw call worst-case underestimated by 6–8 calls (cell nodes = 6 batches not 3; status icons add 3–5 more; true worst case ~18–25); (5) F4 ceiling math incorrect for 5-destroy scenario (18,000ms not ~14,600ms); (6) two ACs described in body text but absent from AC table. Plus Rule 14 status overflow had no priority ordering (game-deciding keywords could be hidden). All 7 blockers resolved in-session. OQ-BR-10 also closed (Approach A confirmed).
Prior verdict resolved: Yes — R3 items remain closed; R4 surfaces a new layer (cross-doc OQ closure, API correctness, rate-limit contract, draw call math, missing ACs)

### R4 design decisions made by user
- **Draw call ceiling**: Accept ~18–25 worst-case; update table to reflect reality; keep BR-3 ADVISORY; no architecture change
- **F4 ceiling**: Remove "≤12.6s absolute ceiling" claim; ceiling now scales with destroy count; ≤5s default remains the anchor
- **Recovery retry**: Document NP-43 rate-limit in all 3 recovery paths; accept 30s Lightyear heartbeat as intentional backstop; no retry loops added
- **Status overflow priority**: Define Tier 1 (SHIELDED/TAUNT/STEALTH/IMMUNE always visible) inline in Rule 14; keyword-system.md must add `display_tier` per keyword

### R4 in-session resolutions (7 BLOCKING)
1. SpriteAlphaLens API: `sprite.color.set_alpha(...)` → `sprite.color = sprite.color.with_alpha(lerp_value)` (Bevy 0.18: `Color` has no mutating `set_alpha` method)
2. OQ-BR-06 marked RESOLVED; stale "currently undefined" text removed from Rule 11/EC-RESOLUTION-REVEAL-STUCK, Dependencies table, OQ section; BR-18c + BR-EC-STUCK ungated
3. NP-43 rate-limit note added to EC-SUBSTEP-OOR, EC-RESOLUTION-REVEAL-STUCK, EC-PLACEMENT-STUCK — clarifies 30s heartbeat is intentional backstop when request is rate-limited
4. Draw call table corrected: cell nodes 3→6 batches; status icon row added; worst-case 12–17→~18–25; explanatory note updated
5. F4 theoretical 5-destroy ceiling corrected (~14.6s→18,000ms at ceiling knobs); Player Fantasy absolute ceiling claim removed; F4 Player Fantasy ceiling paragraph rewritten
6. BR-EC-PLACEMENT-STUCK AC added (from EC-PLACEMENT-STUCK edge case); BR-CAMERA-PROJ AC added (from OQ-BR-02)
7. Rule 14 priority tiers added: Tier 1 = SHIELDED/TAUNT/STEALTH/IMMUNE (always visible); Tier 2 = other keywords (overflow candidates); `display_tier` field requirement added for keyword-system.md

### Status disposition
- All 7 R4 BLOCKING items resolved within board-rendering.md and systems-index.md.
- Recommended cluster for R5: simultaneous-reveal collection-window spec (verify `liv-bevy-lightyear` API for newly-replicated detection); F3 minimum cell_width constraint vs sprite_w; AnimQueue staleness after GAME_OVER truncation; BR-RECONNECT-TIME hardware spec.
- OQ-BR-10 RESOLVED (Approach A).
- OQ-BR-01/02/03/05/07/08/09 remain OPEN (unchanged from R3).

---

## Review — 2026-04-30 (R3) — Verdict: NEEDS REVISION → resolved in-session
Scope signal: XL
Specialists: game-designer, systems-designer, performance-analyst, network-programmer, qa-lead, creative-director (senior synthesis)
Blocking items: 14 | Recommended: 9 | Advisory: 3
Summary: Three compounding failure categories surfaced in R3. First, R2 resolutions were never propagated to dependent docs: game-config.md retained dead fog fields and lacked 5 reveal-tween fields; network-protocol.md still missing C2SRequestSnapshot (OQ-BR-06 open since R2). Second, R2 introduced compile-breaking errors: AnimQueue E0201 (field + method with same name `total_duration_ms`) and F4 ceiling arithmetic error (wrote 11,400ms; correct ceiling is 12,600ms — 1.1s above the stated Player Fantasy promise). Third, several architectural invariants were asserted without specifying mechanisms: simultaneous reveal arrival guarantee, HP bar write-conflict freedom, draw-call ceiling under color-tinting, and PendingResolutionScript inverse-stuck path. All 14 blockers resolved in-session. OQ-BR-06 remains the single external gate for R4 approval.
Prior verdict resolved: Yes — R2 items remain closed; R3 surfaces a different layer (cross-doc sync, compile bugs, unspecified invariants)

### R3 design decisions made by user
- **F4 ceiling**: Raise Player Fantasy ceiling to 12.6s (honest about actual worst-case). Ceiling values unchanged.
- **Reveal tween simultaneity**: Collect-then-reveal buffer pattern — 1-frame buffer after S2CPlacementReveal, all tweens fire simultaneously on the next tick. Guarantees simultaneous beat regardless of Lightyear batching.
- **OQ-BR-04**: Resolved as replicated `SpawnRange` component (Economy System replicates it; Board Rendering reads via `Changed<SpawnRange>`). Event-derived approach rejected (bypasses Economy System; breaks under future mechanics).

### R3 in-session resolutions (14 BLOCKING)
1. game-config.md: removed `board_fog_opacity` + `board_fog_lift_ms`; added `board_unit_reveal_tween_ms`, `board_unit_reveal_start_scale`, `board_reveal_timeout_ms`, `board_obj_id_reconnect_timeout_ms`, `board_obj_reveal_anim_ms`.
2. AnimQueue E0201: renamed field `total_duration_ms_cached`; method `total_duration_ms()` preserved as public API.
3. Rule 1: amended — C2SRequestSnapshot exception documented explicitly.
4. F4 ceiling: corrected to 12,600ms ≈ 12.6s (was 11,400ms — arithmetic error).
5. Player Fantasy: ceiling updated to ≤12.6s.
6. Rule 7 (reveal tween): rewritten to collect-then-reveal buffer (1-frame delay, all tweens fire simultaneously on second tick after S2CPlacementReveal). Eliminates Lightyear replication-batching race.
7. EC-PLACEMENT-STUCK (new): PendingResolutionScript inverse timeout added — when PlacementReveal never arrives, trigger C2SRequestSnapshot after resolution_reveal_timeout_ms. GATED AC added.
8. Rule 6: HP bar write-conflict invariant added ("No Animator<Transform> on fill entity scale axis").
9. Rule 5: draw-call breakdown table added — worst-case 12–17 calls under color-tinting; ceiling may be exceeded at full PLACEMENT state.
10. Internal Constants: `UNIT_SPRITE_WIDTH = 48.0_f32` added (source for co-occupancy constraint formula).
11. OQ-BR-04: RESOLVED — replicated SpawnRange component.
12. BR-7: rewritten — collect-then-reveal buffer; Lightyear detection API to be verified against `liv-bevy-lightyear`.
13. BR-17: apply_deferred clarification — two-system despawn+rebuild requires explicit flush or second app.update() verify.
14. BR-19: poison-entity technique added (insert each banned component to register ComponentId before asserting its absence).
15. BR-SYSTEMSET-ORDER: app.update() pre-run required before schedule graph inspection.
16. BR-18c / BR-EC-STUCK: OQ-BR-06 GATED labels added inline.
17. BR-HP-INVARIANT AC added (BLOCKING — no Animator<Transform> on HP bar fill scale).

### Status disposition
- All 14 R3 BLOCKING items resolved within board-rendering.md and game-config.md.
- One BLOCKING dependency remains EXTERNAL: OQ-BR-06 (C2SRequestSnapshot in network-protocol.md). Status: "Needs Revision (CONDITIONAL pending OQ-BR-06)."
- Re-review recommended after OQ-BR-06 is added to network-protocol.md.

---

## Review — 2026-04-29 (R2) — Verdict: MAJOR REVISION NEEDED → resolved in-session
Scope signal: XL
Specialists: game-designer, systems-designer, qa-lead, ux-designer, performance-analyst, network-programmer, technical-artist, gameplay-programmer (`liv-bevy-018`), creative-director (senior synthesis)
Blocking items: 16 surfaced | Recommended: ~25 | Resolved in-session: 16 BLOCKING + ~12 RECOMMENDED bundled
Summary: Re-review of the GDD after the prior 2026-04-30 in-session pass (see entry below). Eight adversarial specialists surfaced 5 root-cause groups: (1) **Bevy 0.18 API errors in the GDD's own enforcement subsection** — `commands.get_entity` returns `Result` not `Option` (3 sites used `if let Some(...)` which won't compile); `Sprite { color, ..default() }` renders invisible in 0.18 (null `Handle<Image>` default); `Handle<TextureAtlas>` doesn't exist as an asset in 0.18 (split into `Handle<Image>` + `Handle<TextureAtlasLayout>`); ACs BR-3a/BR-2-ATLAS used the wrong type. (2) **F4 timing budget overshoots Player Fantasy ceiling** when objective reveals are added (worst case 11.1s vs 8.5s claimed). (3) **F2 floating-point boundary bug** (`3/10` as f32 = 0.29999... silently classified as Red instead of Yellow) + threshold-inversion vector (no `green > red` validator). (4) **`C2SRequestSnapshot` contract referenced by 4 recovery paths but absent from `network-protocol.md`** (verified by direct grep). (5) **Status-effect visual contract missing** despite Overview promise that "status effects must attach visibly". creative-director synthesis: not implementable as-written. Friend-game scope absorbs ~1/3 of blockers (testability rigor, perf certification, accessibility) but not API correctness, protocol contract gaps, or pillar honesty.
Prior verdict resolved: Yes — prior 2026-04-30 entry's blockers stay closed; this R2 surfaces a different layer of issues (API correctness, formula bugs, cross-doc gaps).

### R2 design decisions made by user
- **F4 ceiling**: Raise to 11.5s and revise Player Fantasy honestly (NOT cap reveals; NOT add tap-to-skip).
- **Watching-IS-reading**: Honest Player Fantasy revision (NOT add tap-to-accelerate; NOT add lane-stagger).
- **Fog rule**: "Don't hide the board, just don't show opponent units placed before end of placing phase." → Result: fog overlay system removed entirely. Server-side replication filtering (already designed in Rule 8) is the actual hide mechanism; the dramatic reveal beat is now a 250ms scale + alpha tween on each newly-replicated opponent entity.
- **Reconnect timeout**: 5 seconds for `S2CObjectiveIdentities` (with 5s/10s/20s/30s backoff schedule).

### R2 in-session resolutions (16 BLOCKING)
1. Bevy 0.18 API table corrected: `Some` → `Ok` for `get_entity` (3 sites); `Sprite::from_color` for solid-color sprites; `Handle<TextureAtlas>` removed → `Handle<Image>` + `Handle<TextureAtlasLayout>` pattern.
2. Rule 4 Z layers: `Z_FOG` and `Z_SPAWN_HIGHLIGHTS` removed; `Z_OBJECTIVES` = 2.5 added; spawn highlights now `Sprite.color` tint on cell nodes.
3. **Rule 7 fully rewritten** — fog overlay system removed; server-side replication filtering + 250ms reveal tween (custom `SpriteAlphaLens` + `TransformScaleLens`); local player's own units don't reveal-tween.
4. Rule 8 ghost: sprite construction updated to atlas-frame + `Sprite.color` tint; reveal tween mention added on placement reveal.
5. Rule 11 reconnect: 5s `S2CObjectiveIdentities` timeout + backoff added; cache-clear-on-reconnect made explicit; snapshot phase-content invariant documented as cross-doc dep.
6. **Rule 13 (NEW)** SystemSet ordering: `BoardRenderSet { ReadMessages, ResolveStateMachine, SpawnEntities, ScheduleTweens, UpdateHpBars, TickAnimations }` with `chain()` ordering.
7. **Rule 14 (NEW)** Status effect visual contract: top-right of unit, 16×16 px, board-elements atlas, max 3 + overflow `+N` badge, no tooltip required.
8. F2 `HP_THRESHOLD_EPSILON = 1e-4`; intake `assert!(red < green)`; examples updated showing 3/10 boundary case fix.
9. F4: reveal tween + objective reveal sequence now in formula; ceiling 11.5s; typical 5.15s; Player Fantasy raised correspondingly with honest watch-time acknowledgment.
10. Player Fantasy: "Watching IS reading" honestly caveated for veteran fatigue; tap-to-skip flagged as out-of-M2 escape hatch; "newcomer learns vocabulary" claim relaxed.
11. Tuning Knobs: `fog_*` removed; `unit_reveal_tween_*`, `resolution_reveal_timeout_ms`, `objective_identities_reconnect_timeout_ms`, `objective_reveal_anim_ms` added; co-occupancy/cell_width constraint clarified.
12. Acceptance Criteria — major rewrite:
    - BR-3a/3b/2-ATLAS rewritten for 0.18 atlas API.
    - BR-3c new (HP bar atlas membership).
    - BR-6/BR-7 rewritten for fog-removal / reveal-tween test.
    - BR-FOG-OPACITY removed (struck through).
    - BR-10 corrected to `if let Ok(...)`.
    - BR-12 type-tightened to u32 millis.
    - BR-13 reveal-tween-aware.
    - BR-18 split into a (blocking sub-state), b (happy path), c (5s timeout + backoff).
    - BR-19 rewritten without `World::inspect_entity` dep — positive component allowlist + explicit banned-`TypeId` enumeration.
    - BR-EC-EARLY split into buffer + consume; pre-pause retained on consumption.
    - BR-EC-STUCK tied to `resolution_reveal_timeout_ms` knob.
    - 9 new ACs: BR-HP-EPSILON, BR-HP-THRESHOLD-INVERT, BR-COOCC-CONSTRAINT, BR-EC-LOBBY-SNAPSHOT, BR-INTERSTEP-PAUSE, BR-STATUS-CONTRACT, BR-STATUS-COOCCUPANCY, BR-RECONNECT-CACHE-CLEAR, BR-SYSTEMSET-ORDER.
    - Implementation notes updated for `App::new().add_plugins(MinimalPlugins)` + `#[should_panic(...)]` pattern.
13. Asset Requirements: fog row removed; `hp_bar_white_pixel_1x2` reserved unit-atlas frame added.
14. Visual/Audio Requirements: "Fog lift" VFX row replaced with "Opponent unit reveal — simultaneous appearance".
15. UI Requirements: Fog overlay row replaced with reveal-tween row.
16. **OQ-BR-06 escalated to BLOCKING** for implementation. `C2SRequestSnapshot` confirmed absent from `network-protocol.md`. Required NP additions enumerated. **External cross-doc revision required before BR-EC-STUCK / BR-18c / BR-24 / EC-RESOLUTION-REVEAL-STUCK / EC-SUBSTEP-OOR can be implemented.**

### Specialist disagreements (surfaced — not silently resolved)
- **game-designer vs ux-designer on watching-IS-reading**: skip mechanism vs honest revision. **User picked honest Player Fantasy revision** (ux-designer's path).
- **ux-designer vs creative-director on EC-INVALID-GHOST** ("stay at last valid" vs despawn): not addressed in R2; preserving prior behavior. Flagged for future polish.
- **qa-lead vs creative-director on BR-Z-LOCAL** (impl-detail vs explicit invariant): kept as-written per creative-director.
- **game-designer F5** (asymmetric fog → draft-pick perception): obsoleted by R2 fog removal.

### Status disposition
- 16 BLOCKING items resolved within `board-rendering.md`.
- One BLOCKING dependency remains EXTERNAL: `network-protocol.md` must define `C2SRequestSnapshot` before recovery paths can be implemented (tracked as OQ-BR-06).
- Recommended: re-review in fresh session **after** `network-protocol.md` revision lands, to confirm cross-doc consistency. Until then, status is "Designed (CONDITIONAL APPROVED pending OQ-BR-06)".

### Cross-doc follow-ups for fresh session
- `network-protocol.md`: define `C2SRequestSnapshot` (OQ-BR-06) — payload, server response invariant, rate-limit, valid-in-all-phases.
- `network-protocol.md` / `round-state-machine.md`: document snapshot phase-content invariant for RESOLUTION reconnect (server holds post-resolution state).
- `keyword-system.md`: provide status-effect icon mapping per Rule 14.
- `combat-resolution.md`: complete `ResolutionEvent` enum variants (OQ-BR-03).
- NP-OQ-3: confirm Lightyear 0.26 reliable channel ordering (affects EC-EVENT-EARLY / EC-PHASE-EARLY).

---

## Review — 2026-04-30 — Verdict: MAJOR REVISION NEEDED → resolved in-session
Scope signal: L
Specialists: game-designer, systems-designer, gameplay-programmer (+liv-bevy-018), network-programmer, performance-analyst, qa-lead, creative-director (senior)
Blocking items: 15 surfaced | Recommended: 12 | Resolved in-session: 24 (some bundled)
Summary: Document is structurally well-organized with all 8 required sections + Visual/Audio + UI Requirements + Open Questions, but cannot be handed to implementation as-written. Three categories of blockers: (1) silent-corruption traps that won't surface in testing — F1 u8 underflow, F2 NaN on hp_max=0, inverted Rule 10 buffer that would silently drop animations on PhaseChanged-before-ResolutionEvent ordering, dead code in Rule 11 reconnect logic; (2) load-bearing data structures never defined — `AnimGroup`, queue storage, sub-step timer mechanism; (3) Player Fantasy "watching IS playing" claim with no design mechanism behind it. All resolved in-session. Player Fantasy reframed honestly to "watching IS reading" + animation defaults cut to defend ≤5s. Added Data Structures subsection, Bevy 0.18 API Contract subsection, formula preconditions + release-loud asserts, EC-coverage of all 4 missing edge cases, BR-3 split, BR-7/BR-20 BLOCKING reclassifications, BR-19 strengthened to ComponentId set equality, 14 new ACs, BR-25 moved to CI lint, sub_step OOR aligned with NP GDD (fatal desync + RequestSnapshot). 5 new Open Questions added (OQ-BR-06 through OQ-BR-10) covering RequestSnapshot C2S contract, trap NP-OQ-2 dependency, Visual Identity Anchor compliance, fake-reveal audio revisit, HP bar poll vs tween implementation choice. F3 / 2v2 retained on M2 critical path per user decision. Re-review recommended in fresh session.
Prior verdict resolved: First review

### Key blockers resolved
- B-1: Player Fantasy "watching IS playing" → reframed to "watching IS reading"; animation defaults cut (sub_step 800→600ms, inter 200→150ms); ceiling tightened (1500→1000, 400→300).
- B-2: F1 u8 underflow on lane=0/cell=0 silently producing units 16,000+ off-screen → precondition + release-loud `assert!`; BR-2b BLOCKING AC.
- B-3: F2 hp_max=0 → NaN scale.x → invisible degenerate sprite → clamp-at-intake (`hp_max.max(1)`) + warn!() pattern; BR-4(f) added.
- B-4: F3 unit_index=2 silent out-of-cell render → precondition + assert!; BR-22b BLOCKING AC.
- B-5: AC BR-13 vs N_groups=0 contradiction → reconciled: pause runs, no Tweens spawned, then transition; F4 + edge case + BR-13 aligned.
- B-6: AnimGroup struct undefined → Data Structures subsection added: `AnimGroup`, `AnimQueue` Resource, `Time<Virtual>`-driven Bevy `Timer`, `PendingPhaseChange`, `PendingResolutionScript`, `ObjectiveIdentityCache`.
- B-7: Rule 10 buffer inverted (didn't protect against PhaseChanged-before-ResolutionEvent) → rewrote to buffer in any RESOLUTION sub-state including `Placement`; last-write-wins; EC-PHASE-EARLY added.
- B-8: Rule 11 reconnect dead code (`if S2CResolutionEvent has been received` was invariantly false) → snapshot during RESOLUTION → always `DraftShop`; animation never replayed; ADR-001 `S2CObjectiveIdentities` re-send required; `Reconnecting` sub-state.
- B-9: 2000ms ResolutionReveal hold had no fallback (player stuck forever) → `RequestSnapshot` C2S contract + EC-RESOLUTION-REVEAL-STUCK + BR-EC-STUCK BLOCKING AC.
- B-10: Bevy 0.18 API gaps → Bevy 0.18 API Contract subsection added (despawn recursive-by-default, `ChildOf` for parenting, `MessageReader<T>` instead of `EventReader`, `q.single()` returns Result, custom SpriteAlphaLens, `set_tweenable` cancel pattern, local-Z=0.1 for HP bar children).
- B-11: Trap rendering NP-OQ-2 dependency → Rule 12 calls out trap face-down dependency; OQ-BR-07 added.
- B-12: BR-3 advisory-only with no automatable proxy → split into BR-3 (ADVISORY GPU), BR-3a (BLOCKING atlas uniqueness), BR-3b (BLOCKING no-Material sentinel).
- B-13: BR-7 / BR-20 misclassified ADVISORY → both reclassified BLOCKING with `Time<Virtual>` injection pattern.
- B-14: BR-19 too vague → strengthened to ComponentId set equality across all 5 standing objective entities + name-pattern query.
- B-15: Sub-step OOR contradiction with NP GDD → adopted NP behavior: fatal desync + RequestSnapshot; BR-24 + EC-SUBSTEP-OOR rewritten.

### Design decisions made by user
- Player Fantasy: Honest reframe to "watching IS reading" (not add-mechanic, not aggressive cut)
- F3 / 2v2: KEEP F3 BLOCKING in M2 (door open for 2v2 with no rework)
- HP edges: Clamp hp_max=0 to 1 + log warning; accept fill=0.0 invisibility (unit despawns synchronously)
- Sub-step OOR: Adopt NP behavior (fatal desync + snapshot)

### Carryover for next pass
- Re-review in fresh session (context heavy)
- OQ-BR-03 ResolutionEvent variants resolution (combat-resolution.md owns)
- OQ-BR-05 unit atlas sizing (art bible owns)
- OQ-BR-06 RequestSnapshot C2S contract (network-protocol.md to add)
- OQ-BR-07 trap rendering depends on NP-OQ-2
- NP GDD update for `RequestSnapshot` C2S message
- Audio-director gate for OQ-BR-09 (dud-thud revisit)
