# Asset Specs - System: Round State Machine

> **Source**: design/gdd/round-state-machine.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-09
> **Status**: 16 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-281 through ASSET-296

---

## Scope Notes

The Round State Machine is a pure server-side phase orchestrator — it owns no art assets directly. It fires phase-change events that downstream systems hook into for all visual and audio output. Per the GDD (Visual/Audio Requirements section): "The RSM owns no art assets. It generates phase-change events that all visual and audio output hooks into."

This spec covers:

- **Phase-transition UI overlays** — the screen elements that announce DRAFT, PLACEMENT, RESOLUTION, and GAME_OVER phase entry to the player. These are triggered by `S2CPhaseChanged` and constitute a distinct visual category not yet owned by the HUD GDD (not yet authored) or the Shop/Auction UI GDD.
- **The `GameOverReason` result screen text styles** — the four outcome variants (`ObjectivesDestroyed`, `Disconnection`, `Draw`, `ResolutionTimeout`) each require a distinct visual treatment.
- **Phase-transition audio stings** — the audio beats that accompany each phase boundary crossing.
- **Shared `RoundState` resource** — the data contract exposing `phase`, `round_number`, and `timer_remaining_ms` for snapshot assembly (Open Question 4 resolution, network-protocol.md Rule 7).

The RSM does **not** own:
- HUD timer bars (Shop/Auction UI — ASSET-219 shared timer bar material; Auction System — ASSET-182)
- Auction panel visuals (Shop/Auction UI)
- Board combat animations (Combat Resolution, Card Animations)
- Class/card selection screens (Game Session System — ASSET-195 through ASSET-214)
- Phase tick SFX already tracked under HUD (ASSET-091)

---

## P0 Assets

### Phase Announcement Overlays

Each phase boundary fires `S2CPhaseChanged`. These are the brief overlay banners that appear at phase entry, informing the player which phase has begun and — where applicable — starting the visible countdown. Per Card Animations Rule C-2: PLACEMENT overlay hard cap is 250 ms; DRAFT and RESOLUTION transition overlays are 350 ms; GAME_OVER is 500 ms.

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-281 | DRAFT_INITIAL Phase Banner | UI Overlay | 480x64 PNG-32 — "DRAFT" lettering, Arcane Gold `#F5C842` on Ink Blue `#1A2D5A`; full-width panel, slides in 350 ms `EaseOutCubic` | atlas_ui_hud | Needed |
| ASSET-282 | DRAFT_SHOP Phase Banner | UI Overlay | 480x64 PNG-32 — "SHOP" lettering, same palette as ASSET-281; slides in 350 ms, auto-exits 350 ms (non-blocking; shop is immediately accessible) | atlas_ui_hud | Needed |
| ASSET-283 | DRAFT_AUCTION Phase Banner | UI Overlay | 480x64 PNG-32 — "AUCTION" lettering, warmer accent (Arcane Gold pulse); slides in 350 ms; dismissed when auction panel fully visible | atlas_ui_hud | Needed |
| ASSET-284 | PLACEMENT Phase Banner | UI Overlay | 480x64 PNG-32 — "PLACEMENT" lettering in bold weight, Prism White `#EEF4FF` on Ink Blue; slides in and exits within 250 ms HARD CAP (per Rule C-2 PLACEMENT budget) — no entry animation on the hand or timer | atlas_ui_hud | Needed |
| ASSET-285 | RESOLUTION Phase Banner | UI Overlay | 480x64 PNG-32 — "RESOLUTION" lettering, Prism White `#EEF4FF` bold; visible 350 ms then dismissed; does not linger into combat animation | atlas_ui_hud | Needed |
| ASSET-286 | Round Number Badge | UI / HUD Element | 48x24 PNG-32 — compact badge displaying round number ("R3", "R9" etc.); Arcane Gold text on Ink Blue chip; persists in HUD corner across all phases; updates value on `S2CPhaseChanged` round_number | atlas_ui_hud | Needed |

### GAME_OVER Result Screen Assets

The RSM broadcasts `S2CGameOver { loser: Option<PlayerId>, round: u32, reason: GameOverReason }`. The result screen renders this payload. Per GDD Rule 14 and Visual/Audio Requirements.

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-287 | GAME_OVER Result Panel Background | UI Background | 640x320 PNG-32 — full Ink Blue `#1A2D5A` panel with Arcane Gold border trim; center-screen modal | Standalone | Needed |
| ASSET-288 | WIN Result Text Style | Runtime Text / Material | "VICTORY" — display font Bold (ASSET-216), 48 px, Arcane Gold `#F5C842`; used when local player is not `loser` in `S2CGameOver` and reason ≠ Draw | N/A | Needed |
| ASSET-289 | LOSS Result Text Style | Runtime Text / Material | "DEFEAT" — display font Bold (ASSET-216), 48 px, Crimson Slate `#C13C38`; used when local player IS `loser` | N/A | Needed |
| ASSET-290 | DRAW Result Text Style | Runtime Text / Material | "DRAW" — display font Bold (ASSET-216), 48 px, Prism White `#EEF4FF`; used when reason = Draw | N/A | Needed |
| ASSET-291 | ResolutionTimeout Result Text Style | Runtime Text / Material | "DRAW — RESOLUTION TIMEOUT" — display font Regular (ASSET-215), 32 px, Prism White; sub-label beneath DRAW result; only fires for reason = ResolutionTimeout | N/A | Needed |
| ASSET-292 | Disconnection Result Sub-label Text Style | Runtime Text / Material | "OPPONENT DISCONNECTED" or "YOU DISCONNECTED" — display font Regular (ASSET-215), 24 px, mid-grey; sub-label beneath WIN/LOSS result for reason = Disconnection | N/A | Needed |
| ASSET-293 | Round Number Result Sub-label Text Style | Runtime Text / Material | "ROUND [N]" — display font Regular (ASSET-215), 20 px, Ivory; displayed beneath WIN/LOSS/DRAW label in result panel; sourced from `S2CGameOver.round` | N/A | Needed |

---

## P1 Assets

### Server-Side Data Types

These are Rust types and resources required for the RSM to expose its state for snapshot assembly (network-protocol.md NP-9, NP-20).

| Asset ID | Name | Category | Format / Location | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-294 | `RoundState` Resource | ECS Resource | `#[derive(Resource)] struct RoundState { phase: RoundPhase, round_number: u32, timer_remaining_ms: Option<u32> }` — readable by snapshot assembly system; `None` when no RSM-owned timer is active (DRAFT_AUCTION, RESOLUTION, GAME_OVER) | N/A | Needed |
| ASSET-295 | `GameOverReason` Enum | Rust Type | `pub enum GameOverReason { ObjectivesDestroyed, Disconnection, Draw, ResolutionTimeout }` — server-side type; replicated to client via `S2CGameOver`; rendered by result screen | N/A | Needed |

---

## Audio Assets

| Asset ID | Name | Category | Format | Naming | Status |
|---|---|---|---|---|---|
| ASSET-296 | PLACEMENT Begin Tension Sting | Audio | OGG Vorbis / WAV master | `sfx_phase_placement_begin.ogg` | Needed |

### Sonic Direction

- **ASSET-296 — PLACEMENT Begin Tension Sting:** fires on `S2CPhaseChanged(PLACEMENT)`. Short, tight, percussive. Communicates "the clock is now running." Must not overlap with the RESOLUTION reveal sting (ASSET-045 — Board Rendering) — PLACEMENT sting plays at PLACEMENT entry; resolve sting plays at RESOLUTION entry. Duration: ≤ 500 ms to clear before any placement action audio.
- **ASSET-091 (HUD) — Phase Transition Tick SFX:** already tracked; plays on phase timer countdowns. Cross-referenced here for completeness — not re-minted.
- **GAME_OVER fanfare and defeat audio:** ownership deferred to the result screen UX design (blocked — ASSET-211/212/213 in game-session-system-assets.md). No new audio IDs minted here for GAME_OVER outcome music.
- **DRAFT phase entry audio:** DRAFT_INITIAL entry sting (ASSET-012) and DRAFT_SHOP entry phrase (ASSET-015) are already tracked in shop-auction-ui-assets.md. Not re-minted.
- **RESOLUTION entry audio:** reveal sting (ASSET-045) already tracked in board-rendering-assets.md. Not re-minted.

---

### Visual Direction

**Phase banner sequencing:**
- ASSET-281 through ASSET-285 use `EaseOutCubic` 350 ms slide-in from the top of screen, except ASSET-284 (PLACEMENT) which is hard-capped at 250 ms for both slide-in and slide-out (Rule C-2).
- At most one phase banner is visible at any time. Previous banner dismisses before the next appears — no overlap.
- ASSET-282 (DRAFT_SHOP) auto-dismisses after 350 ms; does not wait for player input.
- ASSET-283 (DRAFT_AUCTION) dismisses when the auction panel is fully visible (auction panel slide-in completes).
- During RESOLUTION (ASSET-285), no further banners appear until the phase ends.

**GAME_OVER result panel:**
- ASSET-287 panel fades in over 400 ms `EaseOutCubic` (within GAME_OVER 500 ms budget per Rule C-2).
- Text styles (ASSET-288/289/290) appear after panel is fully visible — no simultaneous fade-in of panel and text (Rule C-14: motion soup prevention — ≤ 2 animated regions).
- ASSET-291/292/293 sub-labels appear as instant text-node insertions (no tween) after primary result text is visible.
- No particle effects, no bloom, no animated victory screen elements beyond the panel fade. Art bible: cel-shaded, restraint-first.

**Round Number Badge (ASSET-286):**
- Persistent across all phases. Value updates on each `S2CPhaseChanged` receipt (instantaneous text update, no tween — passive HUD element Rule C-13).
- Positioned in a consistent HUD corner defined by the HUD GDD when authored.

### Technical Notes

- `S2CPhaseChanged` payload: `{ phase, round_number, timer_duration_ms: Option<u32> }`. `None` for DRAFT_AUCTION, RESOLUTION, GAME_OVER. Phase banners must not render an RSM countdown for phases where `timer_duration_ms = None`.
- `S2CGameOver` is a separate broadcast on the reliable channel alongside `S2CPhaseChanged(GAME_OVER)`. Result panel renders from `S2CGameOver.reason` and `S2CGameOver.loser`.
- `ASSET-294 RoundState` resource must expose `timer_remaining_ms: Option<u32>` as a live countdown (ticked by the RSM each frame) so that `S2CGameSnapshot` can include the current remaining time for reconnecting clients (NP-20 contract).
- Phase banner font uses ASSET-215/216 (shared display fonts); no new font assets needed.
- `ASSET-286 Round Number Badge` badge background is a new PNG sprite. Its text content is runtime-generated using ASSET-215 (Regular) or ASSET-216 (Bold) depending on final HUD GDD decision.
