# Board Rendering — Review Log

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
