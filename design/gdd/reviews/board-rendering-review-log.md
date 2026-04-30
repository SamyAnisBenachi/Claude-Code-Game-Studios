# Board Rendering — Review Log

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
