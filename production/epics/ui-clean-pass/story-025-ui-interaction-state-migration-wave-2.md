# Story 025: S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001 -- Bind `interaction_states::*` Tokens to P1 Surfaces

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001
> **Status**: Draft -- future Sprint 18 candidate; NOT activated by this authoring run
> **Layer**: Presentation -- per-surface interaction-state token binding (lobby + hand + shop_auction)
> **Type**: Tech Debt -- per-surface migration (root-cause RC-4)
> **Sprint**: Future Sprint 18 candidate per PROMPT 1180 §6 Lane I.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~0.8d
> **Source audit**: PROMPT 1180 §2 RC-4, §6 Lane I (PROMPT 1198 candidate); cross-refs L-03, S-05, UI-1129-09.

---

## Status / No-Claim Banner

Future Sprint 18 candidate. **No sprint activated.** No claim on release readiness, `QA-COND-0005` Standard-tier completion (affordance is one input, not closure), `QA-COND-0006`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside Lane I / RC-4.

## Problem Class / Prevention Target

**Defect class** (RC-4): `client/src/ui/design_tokens/interaction_states.rs` (story 008 / PROMPT 1009 Done) is **published but unconsumed**. No spawn site reads `HOVER_BG_TINT_ALPHA` / `PRESSED_BG_TINT_ALPHA` / `DISABLED_BG_TINT_ALPHA`. SOURCE-1077-05 noted this. Mirror failures: lobby confirm `?` cards (L-03), shop ready / refresh as text (UI-1129-09), bid increments as baked-`?` PNGs (S-05). HUD pills read as clickable buttons despite being status chips.

**Prevention target**: bind tokens to every P1 button (lobby confirm + create/join, shop ready + refresh, auction bid increments + pass, placement Submit). Use `Interaction` change-detection to swap `BackgroundColor`.

## 1180 Lane Coverage

Owns Lane I:

> | **I — Interaction-state migration (P1 surfaces)** | `client/src/ui/lobby.rs` (E owner), `client/src/ui/shop_auction/mod.rs` (H owner), `client/src/ui/hand/mod.rs` (new owner) | `tests/integration/ui_clean_pass/interaction_state_consumer_coverage_test.rs` (NEW) | **P2** | After E + H + G complete |

Wave 3 — runs after Lanes E (PROMPT 1178), G (PROMPT 1183), H (PROMPT 1182) land on `origin/main`.

## Context

- `client/src/ui/design_tokens/interaction_states.rs` — story 008 module; unconsumed.
- `client/src/ui/lobby.rs:1124-1126` — `LobbyConfirmButton`; no binding.
- `client/src/ui/shop_auction/mod.rs` — bid / ready / refresh / pass spawns (lines drift).
- `client/src/ui/hand/mod.rs:3550-3582` — `PlacementSubmitButton`.

**GDD / ADR**: no change.

**Engine / skills**: Bevy 0.18; `liv-bevy-018`; `Interaction` change-detection canonical.

### Control Manifest Rules

- Required: every P1 button carries (i) `Interaction`; (ii) `BackgroundColor` from `Default` token; (iii) change-detection system swapping between four states on `Interaction.changed()`.
- Required: status chips (HUD pills, lobby status banner, effective-timer readout) do NOT carry `Interaction` (§5 C-4).
- Required: cursor changes on hover; status chips unchanged.
- Forbidden: new RGB literals at spawn sites.
- Forbidden: editing `interaction_states.rs` (consume-only).

## Story Classification

**Integration**.

## Dependencies and Parallelism

### Prerequisites (BLOCKING)

- Lane E (PROMPT 1178), Lane G (PROMPT 1183), Lane H (PROMPT 1182) landed on `origin/main`.

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Stories 020 / 021 / 022 / 023 / 024 / 026 / 027 | YES | Disjoint. |
| Active PROMPTs 1178 / 1182 / 1183 | NO | This row's prerequisites. |
| Active PROMPTs 1187 / 1188 | YES | Different surfaces / docs. |

Most parallel-conflict-prone Lane I row (spans lobby + hand + shop_auction).

## Acceptance Criteria

- [ ] AC1 -- Lobby confirm CTA carries token binding (L-03 resolved).
- [ ] AC2 -- Lobby create / join carry token binding.
- [ ] AC3 -- Shop ready + refresh carry token binding.
- [ ] AC4 -- Auction bid (3 variants) + pass carry token binding (S-05 baked-`?`-PNG cleanup).
- [ ] AC5 -- Placement Submit carries token binding.
- [ ] AC6 -- Status chips do NOT carry `Interaction`: `grep -rn "Interaction" client/src/ui/hud/` shows no pill spawn carries it. Lobby status banner + effective-timer readout same.
- [ ] AC7 -- Cursor changes on hover for P1 buttons; status chips unchanged.
- [ ] AC8 -- Token module unchanged: zero diff on `interaction_states.rs`.
- [ ] AC9 -- `interaction_state_consumer_coverage_test.rs` (NEW) asserts `Interaction` + `BackgroundColor`-from-`Default` + driving `Interaction` through `Hovered` / `Pressed` swaps `BackgroundColor` for each P1 surface.
- [ ] AC10 -- No new RGB literals at consumer spawn sites.
- [ ] AC11 -- `liv-bevy-018` activated.
- [ ] AC12 -- Cargo resource policy applied.
- [ ] AC13 -- No accept-risk closure; `QA-COND-0005` not advanced.
- [ ] AC14 -- Sprint disposition preserved.
- [ ] AC15 -- Worker branch scope contained; slug `work/s18-ui-interaction-state-migration-wave-2`.

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/lobby.rs` | Bind confirm / create / join + change-detection system. |
| `client/src/ui/shop_auction/mod.rs` | Bind ready / refresh / bid (3) / pass. |
| `client/src/ui/hand/mod.rs` | Bind Placement Submit. |
| `tests/integration/ui_clean_pass/interaction_state_consumer_coverage_test.rs` (NEW) | AC9. |
| `client/src/ui/mod.rs` | Register change-detection system in `Update` set if needed. |

### Forbidden files

- `client/src/ui/design_tokens/interaction_states.rs`.
- `client/src/ui/hud/**`, `client/src/ui/settings/**`.
- Server, shared, ADRs, sprint / state / QA files.

## Worker Contract

1. Verify prerequisites (1178 / 1182 / 1183 landed). BLOCK + relay if not.
2. Worktree slug `work/s18-ui-interaction-state-migration-wave-2`.
3. Read story + PROMPT 1180 §2 RC-4 + §5 C-4 + §6 Lane I.
4. Activate `liv-bevy-018`.
5. Cargo resource policy env vars.
6. Targeted tests only.
7. Push worker branch only.
