# Story 009: CI boundary enforcement (no direct S2C subscription in card_animations/)

> **Epic**: Card Animations
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Card Animations is a terminal node — it consumes only intra-client domain events via `MessageReader<T>`, never `S2C*` Lightyear messages directly. Direct S2C subscription in `card_animations/` is classified BLOCKER-severity in GDD Rule C-14 because it would bypass the SystemSet ordering invariant ("game state updated before animation starts") and violate the architectural boundary.

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: LOW
**Engine Notes**: No engine API in this story — CI grep check only. `EventReader<T>` does not exist in Bevy 0.17+; the violation pattern to detect is `MessageReader<S2C` (Bevy-internal S2C readers). `MessageReceiver<S2C*>` (Lightyear inbound) would also be a violation — add that to the grep pattern if Lightyear message types adopt an `S2C` prefix in `card_animations/`.

**Control Manifest Rules (Presentation Layer)**:
- Required: CI `grep -rn "MessageReader<S2C" src/card_animations/` exits with code 1 (no matches) on every merge to main. `CardAnimationsPlugin` registers `MessageReader<T>` only for intra-client domain event types (non-`S2C*` names).
- Forbidden: `MessageReader<S2C*>` in any file under `src/card_animations/`. Direct S2C subscription is BLOCKER-severity per GDD Rule C-14.
- Guardrail: CI must enforce on every merge to main; ADVISORY until CI established; auto-promotes to BLOCKING once CI green.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [x] **CA-14** — GIVEN the CI pipeline runs on every merge to main, WHEN `grep -rn "MessageReader<S2C" src/card_animations/` is run against the repository, THEN exit code is 1 (no matches found). Story is not Done until this CI step exists in the pipeline configuration AND the pipeline passes. Note: `EventReader<S2C` half of the original pattern dropped (QA lead recommendation) — `EventReader<T>` does not exist in Bevy 0.17+ and would never match. ADVISORY until CI established on main; auto-promotes to BLOCKING once CI is green. **[ADVISORY → BLOCKING when CI established]**

---

## Implementation Notes

*Derived from GDD Rule C-10, C-14 and ADR-021 boundary contract:*

1. **CI step implementation:** Add a grep check to the CI pipeline (GitHub Actions `.github/workflows/ci.yml` or equivalent):
   ```yaml
   - name: Check Card Animations S2C boundary
     run: |
       if grep -rn "MessageReader<S2C" src/card_animations/ 2>/dev/null; then
         echo "VIOLATION: direct S2C subscription in card_animations/ — see GDD Rule C-14"
         exit 1
       fi
   ```
   Exit code 0 from the `if` branch = violation found = CI FAIL. No matches = `grep` returns exit 1 = `if` not entered = CI PASS.

2. **Architectural rationale (GDD Rule C-10):** Card Animations consumes ~15 intra-client domain event types. All `S2C*` Lightyear messages are consumed upstream (Board Rendering, Hand UI, Shop/Auction UI), which emit narrow domain events. Adding `MessageReader<S2CAuctionBidAccepted>` etc. to `card_animations/` would bypass the SystemSet ordering invariant enforced by `CardAnimationsSet::React.after(BoardRenderSet::ScheduleTweens)`.

3. **Pattern scope (QA lead clarification):** Pattern narrowed to `MessageReader<S2C` only. `EventReader<T>` does not exist in Bevy 0.17+ and would never match in this codebase. If `MessageReceiver<S2C*>` (Lightyear outbound-type confusion) is also a risk, extend the grep to cover that pattern.

4. **ADVISORY → BLOCKING promotion:** Story can be closed as Done before CI pipeline exists (criterion is ADVISORY). The CI pipeline entry itself is the true completion signal — once CI is green on main, the criterion auto-promotes to BLOCKING for all future merges.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001](story-001-plugin-scaffold-custom-lenses.md): Plugin scaffold (this story enforces the boundary post-implementation across all other stories)

---

## QA Test Cases

*Written by qa-lead at story creation.*

**CA-14 — CI boundary: no direct S2C subscription in card_animations/**

Manual check: CI grep step passes on every merge to main
  - Setup: CI pipeline configured with grep step targeting `src/card_animations/`; command: `grep -rn "MessageReader<S2C" src/card_animations/`
  - Verify: Exit code 1 (no matches) on all passing CI runs; exit code 0 (match found) causes CI failure and blocks the merge
  - Pass condition: Every merge to main passes this check. Exit code 0 = violation added by a developer = BLOCKING, merge rejected.

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- Integration: CI pipeline configuration entry (e.g., `.github/workflows/ci.yml` grep step present and passing on main)

**Status**: [x] Created in `.github/workflows/tests.yml`; local boundary grep passed against `client/src/card_animations/`

---

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-custom-lenses.md) must be DONE (`src/card_animations/` module exists for grep to run against)
- Unlocks: None

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 1/1 passing. CA-14 is covered by the `Check Card Animations S2C boundary` step in `.github/workflows/tests.yml`.
**Deviations**: Advisory only - story references `TR-CAN-001`, which currently maps to CA-1 rather than CA-14 in `docs/architecture/tr-registry.yaml`. Advisory only - current GDD text still mentions legacy `EventReader<S2C` and `src/card_animations/`; the implemented CI check targets the real `client/src/card_animations/` path and checks both `MessageReader` and Lightyear `MessageReceiver` S2C subscriptions.
**Test Evidence**: Integration CI config at `.github/workflows/tests.yml`; local grep found no `Message(Reader|Receiver)<S2C` matches in `client/src/card_animations/`. CI was not waited on per instruction.
**Code Review**: Skipped - Lean mode.
