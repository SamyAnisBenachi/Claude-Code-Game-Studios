# Asset Specs - System: Objective System

> **Source**: design/gdd/objective-system.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-09
> **Status**: 14 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-267 through ASSET-280

---

## Scope Notes

The Objective System is a server-side authority system — it owns data state (HP, identity, destruction facts/counters) but does not own most of its visual output. Visual delegates are:

- **Static objective sprites** (intact, revealed, fake crack): Board Rendering — ASSET-029, ASSET-030, ASSET-031
- **Objective destruction VFX** (Prism White overlay frames, lane gold flood, fake question dissolve): Combat Resolution — ASSET-150, ASSET-151, ASSET-152
- **Objective HP pip HUD display**: HUD — ASSET-088 and downstream HUD GDD (not yet authored)
- **Objective attack ring**: Board Rendering — ASSET-041
- **Objective destruction audio**: Board Rendering — ASSET-047, ASSET-048

This spec owns the assets the Objective System *uniquely* introduces that no other system covers:

1. The **reveal-moment visual beat** — the mandatory 500 ms hold and the identity-reveal animation that fires *after* HP reaches 0. This beat is mechanically defined in the Objective System GDD (Reveal Moment section) and must be treated as its own two-step visual sequence distinct from the destruction VFX owned by Combat Resolution.
2. The **HUD identity indicator** visible to the owning player only — the real/fake per-lane marker in the owner's own objective row.
3. The **Sang Méprise reveal marker** — the temporary visual applied to the opponent's objective slots when `S2CSangMepriseReveal` fires (distinct from Sang Méprise's own class-system assets ASSET-120/121, which mark the Sang Méprise card state itself).
4. **Objective identity replication data types** — the Lightyear message types for the ADR-001 unicast architecture.

---

## P0 Assets

### Reveal-Moment Visual Beat

The GDD mandates a two-beat animation: HP-reaches-0 (beat 1) and identity-reveal (beat 2) with a minimum 500 ms hold between them. This is the primary emotional payload of the system. Board Rendering fires ASSET-150 for the destruction VFX (beat 1 side); the identity reveal (beat 2) is a distinct event.

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-267 | Objective Real Identity Reveal Frame | VFX / Overlay | 64x96 PNG-32 — positioned over objective slot; Arcane Gold `#F5C842` border flash, "REAL" glyph appears | atlas_vfx | Needed |
| ASSET-268 | Objective Fake Identity Reveal Frame | VFX / Overlay | 64x96 PNG-32 — positioned over objective slot; Crimson Slate `#C13C38` desaturation wash, question-mark dissolves (distinct from ASSET-152 which is the destruction animation) | atlas_vfx | Needed |
| ASSET-269 | Objective Reveal Hold Backdrop | Material / UI Overlay | Runtime tint material — dims non-destroyed lanes at 30% opacity during 500 ms hold window to focus attention on the revealed slot | N/A | Needed |

### Visual Direction — Reveal Beat Sequencing

- **Beat 1 (HP reaches 0):** Objective sprite enters destroyed state; ASSET-150 Prism White overlay fires (owned by Combat Resolution).
- **Hold (≥ 500 ms):** Board is held in post-destruction state. ASSET-269 dims surrounding board elements.
- **Beat 2 (Identity reveal):** ASSET-267 or ASSET-268 fires — the actual real/fake identity becomes visible. Must be a distinct visual event from Beat 1 (not a continuous animation from the overlay).
- When `ObjectiveDestroyed` events arrive in a batch (multiple objectives), reveals sequence in ascending lane order per F1 (card-animations stagger contract).
- If Sang Méprise was active this RESOLUTION, suppress the surprise reveal animation (Beat 2) for any objective whose identity was already made visible by Sang Méprise — no visual event fires for a non-surprise.

### Identity Indicator (Owner-Visible HUD Element)

The owning player sees which of their 5 objective slots are real vs. fake at all times. This is distinct from the HP display (shared to both players) and from the reveal animation (fires on destruction).

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-270 | Objective Owner Real Indicator Glyph | UI / HUD | 12x12 PNG-32 — solid Arcane Gold `#F5C842` square or diamond; placed in owner's objective row per-lane | atlas_ui_hud | Needed |
| ASSET-271 | Objective Owner Fake Indicator Glyph | UI / HUD | 12x12 PNG-32 — hollow Crimson Slate `#C13C38` ring or dashed square; placed in owner's objective row per-lane | atlas_ui_hud | Needed |
| ASSET-272 | Objective Owner Destroyed Slot Marker | UI / HUD | 12x12 PNG-32 — X mark in mid-grey; replaces real/fake glyph once the slot is destroyed (RESOLUTION-end sync) | atlas_ui_hud | Needed |

### Sang Méprise Temporary Reveal Marker

When `S2CSangMepriseReveal` fires, the opponent's client temporarily displays objective identities for the current RESOLUTION only. This is distinct from ASSET-120/ASSET-121 (the Sang Méprise card's own class-system VFX) — those mark the Sacrier ability activation; these mark the resulting temporary visibility state on the objective slots.

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-273 | Sang Méprise Slot Reveal Tint — Real | Material / Runtime Tint | Runtime overlay on objective slot entity — Arcane Gold `#F5C842` border pulse, 2 px, applied for RESOLUTION duration | N/A | Needed |
| ASSET-274 | Sang Méprise Slot Reveal Tint — Fake | Material / Runtime Tint | Runtime overlay on objective slot entity — Crimson Slate `#C13C38` border pulse, 2 px, applied for RESOLUTION duration | N/A | Needed |

### Network / Data Types (ADR-001 Architecture)

Per ADR-001, `ObjectiveIdentity` is not a replicated ECS component. The server sends it as a targeted unicast. The message types are deliverables tracked here.

| Asset ID | Name | Category | Format / Location | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-275 | `S2CObjectiveIdentities` Message Type | Rust / Lightyear S2C | `#[derive(Message)] struct S2CObjectiveIdentities { identities: Vec<(LaneId, bool)> }` — sent to owning player at DRAFT_INITIAL and on reconnect | N/A | Needed |
| ASSET-276 | `S2CSangMepriseReveal` Message Type | Rust / Lightyear S2C | `#[derive(Message)] struct S2CSangMepriseReveal { identities: Vec<(LaneId, bool)> }` — sent to opponent only; client clears at RESOLUTION end | N/A | Needed |
| ASSET-277 | `HiddenObjectives` Server Resource | Rust / ECS Resource | `#[derive(Resource)] struct HiddenObjectives` — non-replicated server resource holding authoritative real/fake identities; never sent to client except via S2CObjectiveIdentities unicast | N/A | Needed |
| ASSET-278 | `ObjectiveDestroyed` Message Type | Rust / Lightyear S2C | `#[derive(Message)] struct ObjectiveDestroyed { target_player_id: PlayerId, lane: LaneId, was_fake: bool }` — batched broadcast at RESOLUTION-end sync; both players receive | N/A | Needed |

---

## P1 Assets

### Objective HP Display Components

HP numbers are displayed as world-space text on the board (both players see opposing HP). The text style is derived from the shared display font (ASSET-215/216) — no new font is minted. Tracked here for configuration completeness.

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-279 | Objective HP Number Text Style | Runtime Text / Material | Shared display font (ASSET-215/216), Regular weight, 18 px, Prism White `#EEF4FF` — world-space, centered above objective slot; no float animation (static display) | N/A | Needed |

---

## Audio Assets

| Asset ID | Name | Category | Format | Naming | Status |
|---|---|---|---|---|---|
| ASSET-280 | Objective Identity Reveal Sting | Audio | OGG Vorbis / WAV master | `sfx_objective_identity_reveal.ogg` | Needed |

### Sonic Direction

- **ASSET-280** is the primary audio moment of the entire Objective System. It fires at Beat 2 (identity reveal), not at HP-reaches-0. The distinction matters: Beat 1 (destruction) already has audio from Board Rendering (ASSET-047/048 — real and fake objective destruction SFX). Beat 2 must be audibly distinct — it is the resolution of ambiguity, not the destruction itself.
- Real reveal: resolving, slightly ascending tone. "Confirmed threat, confirmed success."
- Fake reveal: surprising, slightly ironic or hollowed tone. "The trap worked / I was deceived."
- Audio cue at Beat 2 is **required** per Objective System GDD (Reveal Moment section): "Audio cue at the reveal beat is required — this is the primary audio moment of the entire system."
- When Sang Méprise is active and the reveal is non-surprising (identity already known), ASSET-280 does not fire — the "surprise reveal" audio is suppressed per the same rule as the visual suppression.

---

### Visual Direction

- ASSET-267 and ASSET-268 must be visually distinct enough that the reveal reads at a glance — the player must not need to read a label. Arcane Gold (real) vs Crimson Slate desaturation (fake) encode the meaning via palette.
- ASSET-270/271 owner glyphs attach to the owner's HUD objective row, not to the board-space objective entity. They persist across rounds (always visible to the owner) and are only removed when the slot is destroyed (replaced by ASSET-272).
- ASSET-273/274 Sang Méprise tints are transient — they appear on RESOLUTION entry (on `S2CSangMepriseReveal` receipt) and are removed at RESOLUTION end. They must be visually softer than ASSET-267/268 (the authoritative reveal) to prevent the player from reading the Sang Méprise tint as a destruction event.
- No glow/bloom on any objective asset. All effects are flat color fills and outline pulses per art-bible cel-shaded contract.
- The 500 ms minimum hold (Beat 1 → Beat 2) is enforced by the Board Rendering / Card Animations system; art assets do not control timing.

### Technical Notes

- `ObjectiveDestroyed` (ASSET-278) payload includes `target_player_id` to disambiguate which player's objective was destroyed in the case where both players lose an objective in the same RESOLUTION.
- `S2CSangMepriseReveal` (ASSET-276) is a one-shot unicast not included in `S2CGameSnapshot`. Reconnecting clients mid-RESOLUTION will not receive it — Network Protocol GDD must address this gap.
- Board Rendering clears the objective slot visual on receiving `ObjectiveDestroyed` (Rule 9 of Objective System GDD). ASSET-267/268 fire as a separate event at RESOLUTION-end sync, after the 500 ms hold. The two animations must not overlap.
- Reuse ASSET-047 for real objective destruction audio (Beat 1 side). Reuse ASSET-048 for fake objective destruction audio (Beat 1 side). ASSET-280 is Beat 2 only.
