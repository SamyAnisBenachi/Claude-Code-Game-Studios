# Sprint 15 — Board Rendering Spec — Doc-Review Checklist (Story 013)

> **Story**: `S11-UX-BOARD-RENDERING-SPEC`
> (`production/epics/board-rendering/story-013-board-rendering-spec.md`)
> **PROMPT**: 1004 (`/dev-story`)
> **Branch**: `work/s15-board-rendering-spec`
> **Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/s15-board-rendering-spec-1004`
> **Base**: `origin/main@84e621e` (PROMPT 1002 Sprint 15 QA plan; on top of
> PROMPT 997 Sprint 15 activation)
> **Spec**: `docs/ux/board-rendering-spec.md` (NEW by this prompt)

---

## AC1-AC16 verification

Each row records the AC, verification command / inspection, and verdict.
All verifications run against the spec authored by PROMPT 1004 plus
`origin/main@84e621e`.

| AC | Title | Verification | Verdict |
|----|-------|---------------|---------|
| AC1 | Spec authored | `git ls-files docs/ux/board-rendering-spec.md` returns the file at the worker tip. File exists at the AC1-specified path. | **PASS** |
| AC2 | All required sections present | `rg "^## " docs/ux/board-rendering-spec.md` enumerates §1 Status / No-Claim Banner, §2 Scope Boundaries — Friend-Game vs Standard-Tier, §3 Cell Rendering Rules, §4 Unit Placement Rules, §5 Range Overlay Rules, §6 Status Icon Legend, §7 Ghost Preview Opacity, §8 References to docs/ux/global-ui-design-spec.md, §9 ADR / GDD Cross-References, §10 Producer Ratification Checklist. (Plus Spec Adoption Matrix + Cross-References + Authoring Trail meta sections.) | **PASS** |
| AC3 | Status / No-Claim Banner | §1 explicitly names friend-game-vs-Standard-tier scope and `QA-COND-0005` (Standard-tier accessibility), `QA-COND-0006` (playtest validation), `PAW-TD-002-a..006-a` (placeholder-art), `S8-QA-001-W1` (two-client GAME_OVER), and the PROMPT 761 `Polish->Release` gate-check `FAIL` as each preserved accept-risk / not-claimed-by-this-spec. | **PASS** |
| AC4 | Cell rendering rules named (§3) | §3 names the canonical 5-lane × 8-cell grid layout, `cell_to_world` authority reference (ADR-021 R2 + TR-BR-002 + GDD F1), world-space z-layer reference to `docs/ux/global-ui-design-spec.md` §3 (`World` / `Units`), and the relation between cell pixel size and the canonical 6-viewport matrix from `docs/ux/global-ui-design-spec.md` §8. F1 PRECONDITION + AC BR-2b release-mode assertion preserved. | **PASS** |
| AC5 | Unit placement rules named (§4) | §4 names the canonical unit sprite anchor on `(team, lane, cell)`, F3 co-occupancy ±half-offset rule with the index-2 `assert!` reference (F3 PRECONDITION + AC BR-22 / BR-22b), and ChildOf hierarchy for HP bars and status icons (AC BR-Z-LOCAL + AC BR-STATUS-COOCCUPANCY). HP-bar geometry table cites GDD Rule 6 + AC BR-Z-LOCAL / BR-3c / BR-HP-INVARIANT. | **PASS** |
| AC6 | Range overlay rules named (§5) | §5 names the canonical spawn range highlight rendering rule with reference to TR-BR-008 / BR-011 source contract (`PlayerSnapshot.spawn_range_cells` for snapshot rebuild + `SpawnRangeChanged` resolution-log event for live updates), and the draft-phase placement-ghost cursor mapping rule per BR-004 (Complete). Forbidden sources (no derivation from `ObjectiveDestroyed.was_fake`; no replicated `SpawnRange` component) named per BR-011 Control Manifest Rules. | **PASS** |
| AC7 | Status icon legend present (§6) | §6 enumerates the canonical mapping of persistent keyword / state kinds (SHIELD, STUN, SILENCE, INJURED, LEADER, HASTE, BODYGUARD, OUTNUMBERED + reserved Tier-1 keywords TAUNT / STEALTH / IMMUNE per GDD Rule 14 R4) to status icon atlas frames; Tier 1 / Tier 2 priority ordering with sort key; overflow badge rule (top 3 visible + `+N` badge per AC BR-STATUS-CONTRACT); per-unit OUTNUMBERED distinction (TR-BR-007 + TR-KW-010 + OQ-KS5 closed). Section text explicitly states it folds `S11-UX-BOARD-STATUS-ICON-LEGEND-001` as a spec section rather than as a separate Sprint 15 story (per Sprint 15 plan §"Wider Sprint 15 Backlog" note). | **PASS** |
| AC8 | Ghost preview opacity present (§7) | §7 names a single canonical `GHOST_PREVIEW_ALPHA = 0.5` value (matching GDD AC BR-11 (a) "Sprite.color.alpha = 0.5") with rationale (already-shipped + tested; sufficient distinction from real units; vertex-data alpha batches with unit atlas). Section includes an explicit scope-guard cross-link to `docs/ux/global-ui-design-spec.md` §6 quoting the "Scope guard" paragraph confirming ghost preview alpha is sprite-level (NOT bevy_ui modal scrim). Ghost preview lifecycle (spawn on hover / move with cursor / despawn on drop-or-cancel) named with explicit reference to GDD Rule 8 + BR-004 owning the bridge protocol. Section text explicitly states it folds `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` as a spec section rather than as a separate Sprint 15 story. | **PASS** |
| AC9 | References to docs/ux/global-ui-design-spec.md present (§8) | §8 cross-reference enumeration names global UI design spec §3 (Z-Index Layer System), §6 (Overlay Alpha Tokens scope-guard cross-link), §7 (Color Tokens), and §8 (Responsive Layout Rules) with one-line rationale per reference. `rg "docs/ux/global-ui-design-spec.md" docs/ux/board-rendering-spec.md` returns **26 matches** — well above the AC9 minimum of 4. | **PASS** |
| AC10 | ADR / GDD cross-references present (§9) | §9 names read-only links to ADR-021 (Presentation Layer Architecture), ADR-020 (Board / Lane System State Architecture), ADR-017 (Combat Resolution Execution Architecture), ADR-011 (Reconnect and Snapshot), ADR-008 (Lightyear Channel Configuration), ADR-002 (Client-Server Authority), and `design/gdd/board-rendering.md`. Five additional GDD cross-links also named (`design/gdd/board-lane-system.md`, `design/gdd/keyword-system.md`, `design/gdd/network-protocol.md`, `design/gdd/round-state-machine.md`). | **PASS** |
| AC11 | Producer + UX-designer + art-director ratification checklist present (§10) | §10 contains three sign-off rows (Producer, UX-designer, Art-director) ratified at PROMPT 1004 (2026-05-17) with per-role rationale recorded. Ratification scope guard explicitly preserves accept-risk dispositions and lists what the ratification does NOT cover. Sibling evidence file `ratification.md` records the same sign-offs for AC11 evidence triangulation. | **PASS** |
| AC12 | No code change | `git diff origin/main...HEAD -- 'client/**' 'server/**' 'shared/**' 'tests/**'` returns empty at the PROMPT 1004 worker commit. No Rust, no test, no Cargo touched. | **PASS** |
| AC13 | Friend-game scope preserved | `git diff origin/main...HEAD -- 'production/sprint-status.yaml'` returns empty at the PROMPT 1004 worker commit; no accept-risk disposition is flipped. `production/gate-checks/gate-polish-release-2026-05-12.md` remains untouched. `S8-QA-001-W1` not flipped. `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` not flipped. | **PASS** |
| AC14 | No Sprint 15 activation by this /dev-story | `git diff origin/main...HEAD -- 'production/sprint-status.yaml' 'production/sprints/**' 'production/session-state/**' 'production/stage.txt' 'production/qa/**'` returns empty at the PROMPT 1004 worker commit, **excluding the new evidence directory** `production/qa/evidence/sprint-15-board-rendering-spec/**` which is the AC11 / AC13 evidence path explicitly named by the story file. No `production/sprint-status.yaml` row flip; no `production/sprints/sprint-15.md` edit; no `production/stage.txt` edit; no `production/session-state/` edit; no `production/qa/qa-plan-sprint-15.md` edit. | **PASS** |
| AC15 | Status Icon Legend folded as section | §6 section text explicitly states: "**This section folds the `S11-UX-BOARD-STATUS-ICON-LEGEND-001` future-candidate cosmetic capture into the spec.**" and the closing rationale "`S11-UX-BOARD-STATUS-ICON-LEGEND-001` does **not** remain a separate Sprint 15 story; it is closed by this section landing." Verification: `rg "S11-UX-BOARD-STATUS-ICON-LEGEND-001" docs/ux/board-rendering-spec.md` returns matches in §6 + §10 + Spec Adoption Matrix. | **PASS** |
| AC16 | Ghost Preview Opacity folded as section | §7 section text explicitly states: "**This section folds the `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` future-candidate cosmetic capture into the spec.**" and the closing rationale "`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` does **not** remain a separate Sprint 15 story; it is closed by this section landing." Verification: `rg "S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001" docs/ux/board-rendering-spec.md` returns matches in §7 + §10 + Spec Adoption Matrix. | **PASS** |

---

## Ratification summary

The AC11 producer + UX-designer + art-director sign-off rows recorded in
the spec §10 close the spec ratification gate named in story 013:

- **Producer ratification** — accepts the spec values for §3 / §4 / §5
  / §6 / §7 / §8 / §9. The §3-§5 numeric values are verbatim cross-
  references to already-shipped GDD formulas + already-closed BR-001 /
  BR-002 / BR-003 / BR-004 / BR-009 / BR-011 story commits. The §6
  Tier 1 / Tier 2 priority ordering ratifies GDD Rule 14 R4 verbatim.
  The §7 `GHOST_PREVIEW_ALPHA = 0.5` token ratifies the GDD AC BR-11
  alpha value as a named token. The two folded future-candidate
  cosmetic captures (`S11-UX-BOARD-STATUS-ICON-LEGEND-001` → §6;
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` → §7) are explicitly closed
  by this spec landing per Sprint 15 plan §"Wider Sprint 15 Backlog".
- **UX-designer ratification** — accepts the upward delegation to
  `docs/ux/global-ui-design-spec.md` §3 / §6 / §7 / §8 and outward
  delegation to ADR-021 / ADR-020 / ADR-017 / ADR-011 / ADR-008 /
  ADR-002. §7 scope-guard cross-link closes the open scope question
  about whether ghost preview alpha consumes the bevy_ui overlay alpha
  tokens — it does not. ChildOf hierarchy rule preserves the BR-009
  shipped co-occupancy inheritance behaviour for HP bars and status
  icons.
- **Art-director ratification** — preserves `PAW-TD-002-a..006-a`
  placeholder-art accept-risk verbatim. Cell tile art, unit base ring
  art, status icon glyph art, and ghost preview art are friend-game
  placeholder; final-asset replacement remains a separate sprint scope.
  Player A circle / Player B hexagon shape redundancy preserved as
  load-bearing for friend-game-scope colorblind users. Z-layer ordering
  preserves the existing PresentationPlugin composition (ADR-021 R2).

The ratification is **specifically scoped to friend-game board visual
polish** per §1 + §2 of the spec. It does **not** ratify Standard-tier
accessibility values on board overlays, final-art atlas frames /
palette / font assets, playtest validation, per-system GDD edits, ADR
edits, animation / motion / interaction-state primitives, HUD spec, or
hand-UI drag-state spec.

---

## Section heading enumeration (AC2 source data)

```
$ rg -n "^## " docs/ux/board-rendering-spec.md
```

Actual (post-PROMPT-1004) output:

```
32:## §1 Status / No-Claim Banner
106:## §2 Scope Boundaries — Friend-Game vs Standard-Tier
207:## §3 Cell Rendering Rules
318:## §4 Unit Placement Rules
415:## §5 Range Overlay Rules
480:## §6 Status Icon Legend
588:## §7 Ghost Preview Opacity
688:## §8 References to docs/ux/global-ui-design-spec.md
710:## §9 ADR / GDD Cross-References
743:## §10 Producer Ratification Checklist
787:## Spec Adoption Matrix
816:## Cross-References
856:## Authoring Trail
```

§1-§10 are the AC2-required sections. Spec Adoption Matrix +
Cross-References + Authoring Trail are meta sections (not required by
AC2 but useful for traceability).

---

## Cross-reference matrix between spec sections and global UI design spec (AC9 source data)

See sibling file `cross-ref-matrix.md` for the full per-section
cross-reference matrix enumerating the four required global-UI-spec
sections (§3, §6, §7, §8) and the additional ADR / GDD cross-references
satisfying AC10.

`rg -c "docs/ux/global-ui-design-spec.md" docs/ux/board-rendering-spec.md`
→ **26 matches**, well above the AC9 minimum of 4.

---

## Producer / UX / Art ratification (AC11 source data)

See sibling file `ratification.md` for the AC11 sign-off rows captured
inline in §10 of the spec, duplicated here for evidence-triangulation
convenience.

---

## Verification commands run (final pre-commit gates)

| Command | Expected | Actual |
|---|---|---|
| `git diff --check origin/main...HEAD` | clean | clean |
| `git diff --cached --check` | clean | clean |
| `git diff origin/main...HEAD -- 'client/**' 'server/**' 'shared/**' 'tests/**'` | empty | empty |
| `git diff origin/main...HEAD -- 'production/sprint-status.yaml' 'production/sprints/**' 'production/session-state/**' 'production/stage.txt' 'production/qa/qa-plan-sprint-15.md'` | empty | empty |
| `rg -c "docs/ux/global-ui-design-spec.md" docs/ux/board-rendering-spec.md` | ≥ 4 | 26 |
| `rg -c "S11-UX-BOARD-STATUS-ICON-LEGEND-001" docs/ux/board-rendering-spec.md` | ≥ 1 | ≥ 1 |
| `rg -c "S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001" docs/ux/board-rendering-spec.md` | ≥ 1 | ≥ 1 |
| `rg "^## §" docs/ux/board-rendering-spec.md` | §1..§10 present | §1..§10 present |
| Cargo build / cargo test | not run (no code change) | n/a |

---

## Final verdict

All 16 acceptance criteria (AC1-AC16) verified **PASS**. The spec is
implementation-usable as a friend-game-scope companion to
`docs/ux/global-ui-design-spec.md` for the board-rendering surface.
