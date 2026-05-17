# Sprint 15 — Board Rendering Spec — Cross-Reference Matrix (Story 013, AC9 + AC10)

> **Story**: `S11-UX-BOARD-RENDERING-SPEC`
> **PROMPT**: 1004 (`/dev-story`)
> **Spec**: `docs/ux/board-rendering-spec.md` (NEW by this prompt)

This document is the cross-reference evidence file for AC9 (≥ 4 named
references to `docs/ux/global-ui-design-spec.md`) and AC10 (read-only
links to ADR-021 / ADR-020 / ADR-017 / ADR-011 / ADR-008 / ADR-002 +
`design/gdd/board-rendering.md`). It enumerates the per-section cross-
reference map.

---

## AC9 — Global UI Design Spec references

| Global UI spec section | Referenced from | One-line rationale captured in §8 of board rendering spec |
|---|---|---|
| `docs/ux/global-ui-design-spec.md` §3 (Z-Index Layer System) | Board rendering spec §3 (Cell Rendering Rules) + §4 (Unit Placement Rules) + §8 (References) | Canonical `World` (100) / `Units` (200) layer integer values; ADR-021 R2 paint order preserved. |
| `docs/ux/global-ui-design-spec.md` §6 (Overlay Alpha Tokens) | Board rendering spec §7 (Ghost Preview Opacity) + §8 (References) — explicit scope-guard cross-link | `OVERLAY_DIM_ALPHA` / `OVERLAY_SCRIM_ALPHA` are bevy_ui modal-scrim tokens; ghost preview alpha is sprite-level and out of scope for those tokens per global UI spec §6 "Scope guard" paragraph. |
| `docs/ux/global-ui-design-spec.md` §7 (Color Tokens) | Board rendering spec §5 (Range Overlay Rules) + §6 (Status Icon Legend) + §8 (References) | Friend-game palette tokens (`SEMANTIC_SUCCESS`, `SEMANTIC_WARNING`, `SEMANTIC_ERROR`, `ACCENT`, `PRIMARY`, `SECONDARY`) referenced for range overlay tints and status icon tints. Final-art replacement deferred. |
| `docs/ux/global-ui-design-spec.md` §8 (Responsive Layout Rules) | Board rendering spec §3 (Cell Rendering Rules) + §8 (References) | Canonical 6-viewport matrix; world board scales with viewport via camera zoom per §8 "World-space sprites (not bevy_ui)" per-class scaling rule. |

**AC9 quantitative**:
`rg -c "docs/ux/global-ui-design-spec.md" docs/ux/board-rendering-spec.md`
→ **26 matches** (well above AC9 minimum of 4).

---

## AC10 — ADR / GDD cross-references

### ADR cross-references (all read-only)

| ADR | Title | Referenced from spec § |
|---|---|---|
| [ADR-021](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | Presentation Layer Architecture | §3, §4, §8, §9 |
| [ADR-020](../../../docs/architecture/adr-020-board-lane-state-architecture.md) | Board / Lane System State Architecture | §4, §5, §9 |
| [ADR-017](../../../docs/architecture/adr-017-combat-resolution-execution-architecture.md) | Combat Resolution Execution Architecture | §5, §9 |
| [ADR-011](../../../docs/architecture/adr-011-reconnect-snapshot.md) | Reconnect and Snapshot | §5, §9 |
| [ADR-008](../../../docs/architecture/adr-008-lightyear-channel-config.md) | Lightyear Channel Configuration | §5, §9 |
| [ADR-002](../../../docs/architecture/adr-002-client-server-authority.md) | Client-Server Authority | §9 (client-as-view discipline) |

### GDD cross-references (all read-only)

| GDD | Path | Referenced from spec § |
|---|---|---|
| Board Rendering (source of truth) | `design/gdd/board-rendering.md` | §3 (F1 cell_to_world; Rule 3; Rule 4 Z-layer constants; Rule 5 draw-call budget); §4 (F2 HP bar fill; F3 co-occupancy; Rule 6 HP bar invariant; Rule 14 status icon layout); §5 (Rule 8 ghost lifecycle; Rule 11 reconnect rebuild; Rule 12 objective rendering ADR-001 isolation); §6 (Rule 14 R4 display priority); §7 (Rule 8 ghost variant table); §9 (full read-only cross-link to AC table) |
| Board / Lane System | `design/gdd/board-lane-system.md` | §3 (`lane_count = 5`, `cells_per_lane = 8`); §9 |
| Keyword System | `design/gdd/keyword-system.md` | §6 (`display_tier: u8` definitions per keyword; OQ-KS5 closed); §9 |
| Network Protocol | `design/gdd/network-protocol.md` | §5 (TR-NP-014 SpawnRangeChanged ordering; S2CPlacementReveal / S2CGameSnapshot / GhostPlacementChanged message contracts); §9 |
| Round State Machine | `design/gdd/round-state-machine.md` | §9 (BoardRenderState transitions driven by S2CPhaseChanged) |

---

## TR (technical requirement) registry cross-references

| TR | Source registration | Referenced from spec § |
|---|---|---|
| TR-BR-002 | `docs/architecture/tr-registry.yaml:1715` | §3 (BoardLayout single coordinate authority `cell_to_world(lane, cell)`) |
| TR-BR-006 | `docs/architecture/tr-registry.yaml:1751` | §6 (persistent state indicator glyphs Tier-1 priority ordering) |
| TR-BR-007 | `docs/architecture/tr-registry.yaml:1760` | §6 (OUTNUMBERED per-unit; OQ-KS5 closed) |
| TR-BR-008 | `docs/architecture/tr-registry.yaml:1769` | §5 (persistent spawn range highlights snapshot seed + live update contract) |
| TR-KW-010 | `docs/architecture/tr-registry.yaml:1606` | §6 (OUTNUMBERED global board count strict less-than comparison) |
| TR-NP-014 | (referenced in BR-011 Requirement Trace) | §5 (live spawn range updates via `ResolutionEvent::SpawnRangeChanged` in ordered reliable `S2CResolutionEvent`) |

---

## Folded future-candidate cosmetic captures

Per Sprint 15 plan §"Wider Sprint 15 Backlog" reconciliation note:

| Folded slug | Folded into spec § | Section text explicit fold marker |
|---|---|---|
| `S11-UX-BOARD-STATUS-ICON-LEGEND-001` | §6 (Status Icon Legend) | "**This section folds the `S11-UX-BOARD-STATUS-ICON-LEGEND-001` future-candidate cosmetic capture into the spec.**" + closing sentence "`S11-UX-BOARD-STATUS-ICON-LEGEND-001` does **not** remain a separate Sprint 15 story; it is closed by this section landing." |
| `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` | §7 (Ghost Preview Opacity) | "**This section folds the `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` future-candidate cosmetic capture into the spec.**" + closing sentence "`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` does **not** remain a separate Sprint 15 story; it is closed by this section landing." |

Verification:
- `rg "S11-UX-BOARD-STATUS-ICON-LEGEND-001" docs/ux/board-rendering-spec.md`
  → matches in §6 body + §10 ratification rationale + Spec Adoption Matrix row.
- `rg "S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001" docs/ux/board-rendering-spec.md`
  → matches in §7 body + §10 ratification rationale + Spec Adoption Matrix row.
