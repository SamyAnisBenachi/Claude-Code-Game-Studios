# Story 003: S6 Accessibility Disposition and Evidence Register

> **Epic**: Presentation Layer
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-05

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**Primary Sources**:

- `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`
- `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`
- `design/accessibility-requirements.md`
- `design/ux/settings-accessibility.md`
- `production/qa/qa-plan-sprint-6-2026-05-05.md`

**GDD Trace**: N/A - this is a Sprint 6 QA control and evidence-register story,
not a gameplay feature story. No `design/gdd/` requirement owns this register.
The source requirement is the Standard-tier accessibility target in
`design/accessibility-requirements.md`: Standard-tier rows must be implemented,
evidenced, explicitly reclassified, or accepted as risk before QA-COND-0005 can
stop blocking the Production -> Polish gate.

**TR IDs**: N/A. No registered `TR-PRES-*` requirement exists for QA condition
disposition control. This story is traceable to QA-COND-0005, S6-04, the
Sprint 6 QA plan, and ADR-023 timer-accessibility evidence.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
- [ADR-023: Placement Timer Accessibility Authority](../../../docs/architecture/adr-023-placement-timer-accessibility-authority.md)

**ADR Decision Summary**: GSS-008 already implemented the only currently
verified Standard-tier accessibility sub-gap, the server-authoritative
PLACEMENT timer extension from ADR-023. This story records evidence and formal
producer dispositions for every remaining row before any broad Settings,
colorblind, reduced-motion, input-remapping, tutorial, render-calibration, or
audio-control implementation scope expands.

**Engine Notes**: N/A - no engine API or code changes are involved.

**Control Manifest Rules (2026-05-05)**:

- Required: PLACEMENT timer multiplier is server-authoritative, neutral, capped
  at 3x, and frozen at `SessionReady`.
- Required: RSM applies the frozen PLACEMENT timer multiplier and clients use
  server-provided phase duration.
- Forbidden: Never let client-local Settings alter the active multiplayer
  PLACEMENT timer after `SessionReady`.
- Forbidden: Never expose 0.5x as a multiplayer Standard-tier PLACEMENT timer
  value.

---

## Scope

### In Scope

- Update the Sprint 6 Standard-tier accessibility evidence register at the exact
  path `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`.
- Preserve the GSS-008 PLACEMENT timer-extension evidence linkage as
  implemented + evidence attached.
- Build a complete Standard-tier disposition register covering every row listed
  in the row inventory below.
- Force exactly one final disposition per source row:
  - implemented + evidence attached
  - evidence-only required
  - must implement in Sprint 6
  - accepted risk with producer signoff
  - reclassified out of Production -> Polish gate
  - later sprint / blocked dependency
- Add a dependency register for settings/accessibility screen, preference
  persistence, input action registry, audio controls pipeline, render
  calibration approach, and tutorial/help prompt registry.
- Record exact audits, evidence paths, and signoff requirements for every
  accepted-risk or evidence-only row.
- Update `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
  only if every source row has a valid disposition and every required producer
  signoff is present.

### Out of Scope

- No code implementation.
- No Settings or Accessibility UI implementation.
- No colorblind palette or toggle implementation.
- No UI scaling implementation.
- No reduced-motion implementation.
- No input remapping implementation.
- No tutorial persistence or Help registry implementation.
- No brightness or gamma implementation.
- No audio bus or independent volume-control implementation.
- No changes to sprint status, session-state files, asset files, or `AGENTS.md`.

---

## Disposition Register Contract

The evidence register must include a final row table with these columns:

| Column | Requirement |
|---|---|
| Row ID | Stable local ID from the row inventory in this story. |
| Source row | Exact source row label from `design/accessibility-requirements.md` or the Sprint 6 dependency row label. |
| Tier | Standard, Basic baseline, N/A, or dependency. |
| Current evidence | Existing evidence path, audit result, or explicit no-evidence statement. |
| Final disposition | Exactly one of the six allowed dispositions. |
| Required audit or test | Specific audit, command, manual verification, or evidence review needed for the disposition. |
| Producer signoff required | Yes or No. |
| Signoff evidence | Producer name or user-as-producer decision, date, and decision text when required. |
| QA-COND-0005 impact | Blocks closure, does not block closure, or closes sub-gap. |
| Follow-up path | Story, evidence path, backlog item, or dependency row when work is deferred. |

Invalid register states:

- A row has no final disposition.
- A row has more than one final disposition.
- A row uses candidate language as its final state.
- A row requires accepted risk or reclassification but lacks producer signoff.
- A row is removed from the source inventory without an explicit reclassification
  or not-applicable signoff.
- QA-COND-0005 is marked closed while any row remains must implement in Sprint 6
  or later sprint / blocked dependency.

## Source Row Inventory

The evidence register must include all rows below. No row may be silently
dropped.

| Row ID | Source row | Tier | Initial expected disposition path |
|---|---|---|---|
| A11Y-ST-01 | Minimum text size - HUD | Standard | evidence-only required or must implement in Sprint 6 |
| A11Y-ST-02 | Minimum text size - card text | Standard | evidence-only required or must implement in Sprint 6 |
| A11Y-ST-03 | Text contrast - UI on backgrounds | Standard | evidence-only required or must implement in Sprint 6 |
| A11Y-ST-04 | Colorblind mode - Protanopia / Deuteranopia | Standard | reclassification candidate or must implement in Sprint 6 |
| A11Y-ST-05 | Colorblind mode - Tritanopia | Standard | reclassification candidate or must implement in Sprint 6 |
| A11Y-ST-06 | UI scaling | Standard | reclassification candidate or must implement in Sprint 6 |
| A11Y-ST-07 | Motion / animation reduction mode | Standard | reclassification candidate or must implement in Sprint 6 |
| A11Y-ST-08 | Full input remapping | Standard | reclassification candidate or must implement in Sprint 6 |
| A11Y-ST-09 | PLACEMENT timer extension | Standard | implemented + evidence attached through GSS-008 |
| A11Y-ST-10 | Hold-to-press alternatives | Standard | accepted-risk candidate after audit or must implement in Sprint 6 |
| A11Y-ST-11 | DRAFT_SHOP ready signal - retractable | Standard | evidence-only required |
| A11Y-ST-12 | Auction bid buttons - immediate preset commitments | Standard | evidence-only required |
| A11Y-ST-13 | Mana pools: distinct container shapes | Standard | evidence-only required or must implement in Sprint 6 |
| A11Y-ST-14 | PLACEMENT staged disclosure | Standard | evidence-only required or must implement in Sprint 6 |
| A11Y-ST-15 | Tutorial persistence | Standard | reclassification candidate or must implement in Sprint 6 |
| A11Y-ST-16 | Phase label always visible | Standard | evidence-only required |
| A11Y-ST-17 | Gold counter always visible | Standard | evidence-only required |
| A11Y-ST-18 | DRAFT_INITIAL: clear objective | Standard | evidence-only required or must implement in Sprint 6 |
| A11Y-ST-19 | Visual indicators for audio cues | Standard | accepted-risk candidate after audit or must implement in Sprint 6 |
| A11Y-BS-01 | Color-as-only-indicator audit | Basic baseline under Standard target | evidence-only required or must implement in Sprint 6 |
| A11Y-BS-02 | Brightness / gamma controls | Basic baseline under Standard target | reclassification candidate or must implement in Sprint 6 |
| A11Y-BS-03 | Screen flash warning | Basic baseline under Standard target | accepted-risk candidate after audit or must implement in Sprint 6 |
| A11Y-BS-04 | Pause anywhere | Basic baseline under Standard target | reclassification candidate or must implement in Sprint 6 |
| A11Y-BS-05 | Independent volume controls | Basic baseline under Standard target | accepted-risk candidate after audit or must implement in Sprint 6 |
| A11Y-NA-01 | No dialogue / voiced content | N/A in source draft | not applicable after audit with producer signoff |

## Dependency Row Inventory

The evidence register must include a separate dependency table for rows that
block or explain later implementation. These rows are not substitutes for source
row dispositions; they explain why a source row is later sprint / blocked
dependency or reclassified out of the current gate.

| Dependency ID | Dependency row | Required evidence |
|---|---|---|
| A11Y-DEP-01 | Settings/accessibility screen | Identify which source rows require a settings screen and whether they are Sprint 6 gate blockers. |
| A11Y-DEP-02 | Preference persistence | Identify which source rows require persisted preferences and whether localStorage/profile storage is ready. |
| A11Y-DEP-03 | Input action registry | Identify whether full input remapping is blocked by a canonical action registry. |
| A11Y-DEP-04 | Audio controls pipeline | Identify whether Music, SFX, and UI buses or gameplay-critical audio cues ship in this gate. |
| A11Y-DEP-05 | Render calibration approach | Identify whether brightness/gamma depends on renderer, shader, or browser/canvas calibration decisions. |
| A11Y-DEP-06 | Tutorial/help prompt registry | Identify whether tutorial persistence depends on a prompt registry and help content ownership. |

---

## Accepted-Risk Candidate Evidence

Accepted-risk rows require producer signoff. The register must show the audit
that makes accepted risk defensible before the signoff can count.

| Candidate row | Required audit or evidence before signoff |
|---|---|
| Hold-to-press alternatives | Search UX specs and implementation for hold-to-confirm, long-press, press-and-hold, timer-gated button hold, and pointer-held flows. If no shipped hold input exists, evidence may support accepted risk. If any shipped hold input exists, list each flow and either require Sprint 6 implementation of an alternative or record producer accepted risk per flow. |
| Screen flash warning | Audit RESOLUTION, GAME_OVER, combat hit, objective destruction, phase transition, and animation specs for flash/flicker/burst behavior. Evidence must state whether any shipped effect can exceed three flashes per second or create full-screen flash. |
| Independent volume controls | Audit whether Music, SFX, and UI audio buses or gameplay-critical audio content ship in the Production -> Polish gate build. If no audio controls pipeline ships in this gate, producer may accept risk with a later audio-controls dependency row. |
| Visual indicators for deferred audio-only cues | Audit gameplay-critical audio cues and map each cue to a visible backup, a non-shipping audio status, or a deferred implementation dependency. Any shipped audio-only critical cue blocks accepted risk unless producer explicitly signs the exposure. |
| No dialogue / voiced content | Audit narrative, UX, audio, and gameplay docs for voiced dialogue or spoken instructions. If none ships, record not applicable with producer signoff and the evidence terms used for the audit. |

Required signoff fields:

- Producer or user-as-producer name.
- Date.
- Row ID.
- Decision text.
- Reason the risk is acceptable for the Production -> Polish gate.
- Follow-up owner or explicit no-follow-up-needed statement.

## Reclassification Candidate Evidence

Rows below may be reclassified out of the Production -> Polish gate only with
producer signoff. Reclassification must preserve a follow-up path.

| Candidate row | Required reclassification evidence |
|---|---|
| Full colorblind palette/toggles | Explain why shape/icon backups and current color-as-text support are sufficient for this gate, or mark as must implement in Sprint 6. |
| UI scaling 75%-150% | Explain why the gate can proceed without full scale controls, or mark as must implement in Sprint 6. |
| Reduced motion mode | Explain current motion exposure and why reduced-motion controls can move to Polish, or mark as must implement in Sprint 6. |
| Full input remapping | Explain current keyboard/mouse action surface and whether a canonical input action registry blocks implementation. |
| Brightness/gamma controls | Explain whether render calibration is needed for this gate and identify the render calibration dependency. |
| Tutorial persistence / Help registry | Explain whether current tutorial/help content is required for this gate and identify the prompt registry dependency. |
| Pause anywhere | Explain multiplayer safe-phase constraints and whether pause behavior can move to Polish without blocking the current gate. |

Required signoff fields match the accepted-risk section. A reclassified row does
not block QA-COND-0005 closure only after the signoff and follow-up path are in
the evidence register.

---

## Acceptance Criteria

- [ ] The evidence register at `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md` exists and uses the disposition register contract from this story.
- [ ] The register includes every source row from the Source Row Inventory, including the GSS-008 PLACEMENT timer-extension row and the N/A no-dialogue row.
- [ ] The register includes every dependency row from the Dependency Row Inventory.
- [ ] Every source row has exactly one final disposition from the allowed six-value list.
- [ ] No source row uses candidate language as its final disposition.
- [ ] The PLACEMENT timer-extension row links GSS-008 evidence and is marked implemented + evidence attached.
- [ ] Every evidence-only required row lists the exact browser/WASM capture, automated test, source audit, or evidence review needed to move it to implemented + evidence attached.
- [ ] Every accepted-risk row has the required audit result and a complete producer signoff block.
- [ ] Every reclassified row has a complete producer signoff block and a follow-up path.
- [ ] Every later sprint / blocked dependency row links to one or more dependency rows and states whether QA-COND-0005 remains open.
- [ ] The register explicitly lists rows that still block QA-COND-0005 closure.
- [ ] The register explicitly lists rows that no longer block QA-COND-0005 closure and why.
- [ ] `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains unchanged unless every source row has a valid disposition and every required signoff is present.
- [ ] If QA-COND-0005 is updated, the condition remains Open when any source row is must implement in Sprint 6 or later sprint / blocked dependency.
- [ ] If QA-COND-0005 is closed, the closure text states that all rows are implemented/evidenced, reclassified, or accepted risk with producer signoff, and that no unverified Standard-tier blocker remains.
- [ ] The story output does not implement Settings UI, colorblind modes, UI scaling, reduced motion, input remapping, tutorial persistence, brightness/gamma, or audio controls.
- [ ] `git diff --check` passes.

---

## QA Test Cases

- **Complete row inventory**
  - Given: the register is updated
  - When: QA compares it to the Source Row Inventory and Dependency Row Inventory
  - Then: every row ID is present exactly once in the appropriate table.

- **Single disposition per source row**
  - Given: the register source-row table
  - When: QA reviews the Final disposition column
  - Then: every source row uses exactly one allowed disposition and no candidate
    language remains as a final state.

- **Accepted-risk evidence gate**
  - Given: a row is dispositioned accepted risk with producer signoff
  - When: QA reviews the audit and signoff fields
  - Then: the audit result, producer name, date, decision text, acceptability
    reason, and follow-up owner or no-follow-up statement are present.

- **Reclassification evidence gate**
  - Given: a row is dispositioned reclassified out of Production -> Polish gate
  - When: QA reviews the signoff and follow-up fields
  - Then: producer signoff is present and the row has a follow-up path or an
    explicit not-applicable decision.

- **QA-COND-0005 closure guard**
  - Given: QA-COND-0005 is edited
  - When: QA compares the condition file to the register
  - Then: closure occurs only if no source row remains must implement in Sprint 6
    or later sprint / blocked dependency.

## Test Evidence

**Story Type**: Config/Data

**Required evidence document**:

- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`

**Required verification commands and reviews**:

- `git diff --check`
- Manual QA register review against this story's Source Row Inventory.
- Manual producer signoff review for every accepted-risk and reclassified row.
- Manual QA-COND-0005 impact review before editing
  `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`.

**Status**: [ ] Register update not yet completed by this story.

## Dependencies

- Depends on: `production/epics/game-session-system/story-008-placement-timer-multiplier-authority.md` (Complete) for GSS-008 timer-extension evidence.
- Depends on: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` (Open) for condition status and closure rules.
- Depends on: `production/qa/qa-plan-sprint-6-2026-05-05.md` for Sprint 6 S6-04 verification expectations.
- Depends on: ADR-002, ADR-021, and ADR-023 Accepted.
- Unlocks: narrowly scoped accessibility implementation stories, only after the register proves which rows truly need Sprint 6 implementation and which rows are evidenced, accepted risk, reclassified, or dependency-blocked.

## Performance Budget

No runtime performance impact expected - this is docs-only QA evidence and
condition-register work.

## No Open Questions

No unresolved design questions remain for this control story. Producer decisions
are modeled as required story outputs, not as pre-implementation blockers.
