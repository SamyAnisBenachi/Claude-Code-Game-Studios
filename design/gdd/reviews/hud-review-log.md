# Review Log — HUD GDD

## Review — 2026-04-30 (Pass 4) — Verdict: APPROVED
Scope signal: L
Specialists: none (lean mode)
Blocking items: 1 resolved | Recommended: 4 resolved | Nice-to-have: 2 resolved
Summary: Single blocking issue: HUD-01 entity count asserted "exactly 16" but a correct implementation pre-spawns 18 entities — the TextSpan child entities (one per gold label, pre-spawned empty) were not counted. Fix was a two-line AC edit. Recommended fixes: `GoldDisplayState` struct definition in Rule 1 unified with `is_populated: bool` field already present in UI Requirements and Edge Cases; Rule 11 scheduling terminology clarified from "lower priority" to explicit `.before()`; OQ-HUD-01 pre-implementation gate note added (NP GDD must reach Approved before pause overlay can be implemented). Nice-to-have: explicit LOBBY-silent row in audio table; D.1 `u32`→`f32` type conversion note. GDD is otherwise production-ready.
Prior verdict resolved: Yes — Pass 3 blockers (Bevy 0.18 API, Lens conflict, Observer) were fully resolved.

## Review — 2026-04-30 (Pass 3) — Verdict: NEEDS REVISION → Revised in-session
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, ux-designer, ui-programmer, network-programmer, audio-director, creative-director
Blocking items: 14 resolved | Recommended: 25 noted
Summary: Bevy 0.18 API corrections were the primary blocking category: `TextSpan` child entities replace the pre-0.15 `TextSection` API throughout; `BorderRadius` corrected as a `Node` field; `bevy_ui_picking` Cargo feature renamed to `ui_picking` requiring `#[cfg]` gate; `Lens<GoldDisplayState>` option removed due to three-writer conflict with Rule 11 (separate-backing-field pattern mandated). `HudObjectiveUpdate` switched from `EventWriter`/`EventReader` to Bevy Observer (`commands.trigger` / `app.observe`) for guaranteed same-frame delivery. Rule 11 implementation note replaced with a field-split correctness proof (drain-order concern was a non-issue). S2CGoldBroadcast mode-independence contract added explicitly. Rule 13 extended to handle snapshot-during-LOBBY. Three ACs rewritten for testability: HUD-07 (absence assertion now finite-enumerable), HUD-19 (unbounded negative replaced with explicit message sequence + assertions), HUD-23 (entity-level Visibility assertion). Audio GAME_OVER tick exemption made explicit. Creative director verdict: core architecture sound; primary remaining design tension is 300ms tweens during auction conflicting with "never a focal point" fantasy.
Prior verdict resolved: Yes — Pass 2 blockers (Bevy API, Lens conflict, ObserverObjective dual-drain) were the focus of this pass.

## Review — 2026-04-30 (Pass 2) — Verdict: MAJOR REVISION NEEDED → Revised in-session
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, ux-designer, ui-programmer, network-programmer, audio-director, creative-director
Blocking items: 16 resolved | Recommended: 10 noted
Summary: Gold layout redesigned to inline parenthetical `Xg (Yr)` — both gold values remain in top-right but as 2 lines always (prior 4-line ECONOMY_AUCTION stack failed peripheral omniscience per convergent game-designer + ux-designer finding). ObjectiveDestroyed double-drain correctness bug resolved via Board Rendering sole drain → HudObjectiveUpdate Bevy Event re-broadcast. GoldDisplayState backing f32 component specified for bevy_tweening; PickingBehavior API corrected; dot rendering spec'd as Node+border+border_radius; 7 BLOCKING ACs rewritten (HUD-08 server invariant violation, HUD-09 untestable visual weight, HUD-17 weak assertion, HUD-20 wrong story type, HUD-25 boundary underspecified, HUD-26 observation mechanism, HUD-28 now tests single-entity inline format); 3 new ACs added (HUD-29/30/31); 2 new OQs (LANE_MIDPOINT_X sharing, HudObjectiveUpdate location).
Prior verdict resolved: Yes — Pass 1 blockers (diagonal layout, GAME_OVER tiebreak, AC field errors) fully carried forward; Pass 2 added new blockers found by specialist review.

## Review — 2026-04-30 (Pass 1) — Verdict: MAJOR REVISION NEEDED → Revised in-session
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, ux-designer, network-programmer, audio-director, creative-director
Blocking items: 10 resolved | Recommended: 9 noted
Summary: Structural layout failure (diagonal gold zones were physiologically implausible for peripheral omniscience) resolved by consolidating both gold readouts to top-right. Inter-GDD contradiction (S2CGameSnapshot.PlayerSnapshot missing reserved_gold field, required by NP GDD amendment). Rule 10 vs Rule 13 GAME_OVER reconnect tiebreak defined (snapshot always wins, then re-freeze). Seven new BLOCKING ACs added (round counter format, GAME_OVER visibility, HUD root at LOBBY, cold-start placeholder, ObjectiveDestroyed idempotency, FROZEN+snapshot, sub-label entity count). Four AC field name specification errors corrected. OQ-HUD-01 reclassified as gameplay correctness blocker (NP GDD must define S2CSessionPaused before timer-bearing phases can ship); OQ-HUD-02 design-rejected (settings flag recreates screen-share leak).
Prior verdict resolved: No — first review
