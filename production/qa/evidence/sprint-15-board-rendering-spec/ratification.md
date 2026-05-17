# Sprint 15 — Board Rendering Spec — Producer / UX / Art Ratification (Story 013, AC11)

> **Story**: `S11-UX-BOARD-RENDERING-SPEC`
> **PROMPT**: 1004 (`/dev-story`)
> **Spec**: `docs/ux/board-rendering-spec.md` §10 (Producer Ratification Checklist)
> **Date**: 2026-05-17

This file is the AC11 evidence triangulation for the producer +
UX-designer + art-director sign-off captured inline in §10 of
`docs/ux/board-rendering-spec.md`. The sign-offs are reproduced here
verbatim for evidence-directory completeness.

---

## Producer ratification — PROMPT 1004 (2026-05-17)

Spec values are either (a) verbatim cross-references to already-shipped
Tier 0 modules and ADRs (§3 `World` / `Units` z-layers cross-link to
global UI spec §3; §8 cross-references to global UI spec §3 / §6 / §7
/ §8; §9 ADR cross-references all read-only) or (b) ratifications of
values already shipped by the board-rendering epic (§3 `cell_to_world`
cited verbatim from GDD F1; §4 F3 co-occupancy ±half-offset cited
verbatim from GDD F3 with index-2 `assert!`; §4 HP bar geometry cited
verbatim from GDD Rule 6 + AC BR-Z-LOCAL / BR-HP-INVARIANT; §5
spawn-highlight contract cited verbatim from BR-011 closure; §6 status
icon legend cited verbatim from BR-009 closure + GDD Rule 14 R4) or (c)
explicit new tokens (`GHOST_PREVIEW_ALPHA = 0.5` in §7, ratified to be
the canonical token name for the already-shipped GDD AC BR-11 alpha
value).

The two folded future-candidate cosmetic captures
(`S11-UX-BOARD-STATUS-ICON-LEGEND-001` → §6;
`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` → §7) are explicitly closed by
this spec landing per the Sprint 15 plan §"Wider Sprint 15 Backlog"
note. Producer accepts the folding.

Friend-game scope preserved verbatim per §1 + §2; `QA-COND-0005` /
`QA-COND-0006` / `PAW-TD-*-a` / `S8-QA-001-W1` not advanced.

**Verdict**: RATIFIED.

---

## UX-designer ratification — PROMPT 1004 (2026-05-17)

Layout primitives delegate UP cleanly to
`docs/ux/global-ui-design-spec.md` (z-layers, color tokens, overlay
alpha scope-guard, responsive matrix) and OUT cleanly to ADR-021 /
ADR-020 / ADR-017 / ADR-011 / ADR-008 / ADR-002. The §6 status icon
legend Tier 1 / Tier 2 mapping is the same priority ordering shipped by
BR-009; the §7 ghost preview opacity `0.5` is the same alpha already
locked by GDD AC BR-11. The §7 explicit scope-guard cross-link to
global UI spec §6 confirms ghost preview alpha is sprite-level (NOT
bevy_ui modal scrim), closing the open scope question.

Co-occupancy ChildOf hierarchy rule (§4) preserves the BR-009 shipped
invariant that status icons and HP bars inherit the parent unit's
`Transform.translation.x` (including F3 offset) rather than re-centring
on the cell — this is the load-bearing visual correctness behaviour and
the spec ratifies it.

**Verdict**: RATIFIED.

---

## Art-director ratification — PROMPT 1004 (2026-05-17)

`PAW-TD-002-a` … `PAW-TD-006-a` placeholder-art accept-risk preserved
verbatim in §1 Status Banner. Cell tile art (§3), unit base ring art
(§4), status icon glyph art (§6), and ghost preview art (§7) are
friend-game placeholder; final-asset replacement remains a separate
sprint scope.

Player A circle / Player B hexagon base-ring shape redundancy (GDD §Asset
Requirements colorblind-redundancy row) preserved in §4 — shapes are
load-bearing, not just decorative, for friend-game-scope colorblind
users.

Friend-game palette tokens (§5 / §6) cross-reference
`docs/ux/global-ui-design-spec.md` §7 without overriding palette values.

Z-layer ordering (§3 / §4) preserves the existing PresentationPlugin
composition per ADR-021 R2.

**Verdict**: RATIFIED.

---

## Ratification scope guard

The above ratification is **specifically scoped to friend-game board
visual polish** per §1 + §2 of the spec. It does **not** ratify:

- Standard-tier accessibility values on board overlays / status icons /
  ghost previews / cell hit-targets (separate accessibility spec
  required to advance `QA-COND-0005`).
- Final-art atlas frames, palette, font assets (separate sprint scope;
  `PAW-TD-*-a` accept-risk preserved).
- Playtest validation (`QA-COND-0006` accept-risk preserved).
- Per-system GDD edits (read-only cross-link only per §9 of the spec).
- ADR edits (read-only cross-link only per §9 of the spec).
- Animation / motion / interaction-state primitives (owned by ADR-017
  and the Sprint 15 Nice to Have
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` story).
- HUD spec or hand-UI drag-state spec (owned by their respective Sprint
  14 / Sprint 15 stories per §2 Out of Scope of the spec).
- Two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN).
- Polish → Release gate-check retry (PROMPT 761 `FAIL` preserved; no
  retry in scope).

---

## No-claim summary

PROMPT 1004 (this `/dev-story`) authors **only** the doc-only spec and
its evidence directory. It does **not**:

- Activate any Sprint 15 row beyond what PROMPT 997 already activated.
- Flip any `production/sprint-status.yaml` row.
- Modify `production/sprints/sprint-15.md`, `production/stage.txt`,
  `production/session-state/`, or `production/qa/qa-plan-sprint-15.md`.
- Modify any file under `client/`, `server/`, `shared/`, or `tests/`.
- Run `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, or
  `/story-done`.
- Advance the stage from `Polish` to `Release`.
- Retry the PROMPT 761 `Polish->Release` gate-check FAIL.
- Close `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or any
  `PAW-TD-*-a` accept-risk row.

The two folded future-candidate cosmetic captures
(`S11-UX-BOARD-STATUS-ICON-LEGEND-001` → §6;
`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` → §7) are closed **only as
"folded into spec section"** — not as gameplay or final-art closures.
Underlying placeholder-art and accessibility dispositions remain
preserved.
