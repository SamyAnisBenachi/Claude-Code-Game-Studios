# Story 010: Plugin-Registration Audit and Dead-Plugin Sweep

> **Epic**: Playable Client
> **Story ID**: S10-TD-002
> **Status**: Complete
> **Layer**: Tech Debt / Audit
> **Type**: Config/Data
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 10 active

## Context

This story closes out the plugin-registration audit and dead-plugin sweep
surfaced by the PROMPT 545 DRAFT_INITIAL silent-failure breakthrough. Two
server plugins (`CardPoolPlugin` + `KeywordPlugin`) were defined but never
`.add_plugins(...)`-registered in `server/src/main.rs`; that gap was invisible
to `cargo check` and to every per-system unit test in the workspace. The
PROMPT 563 audit enumerated every `pub struct *Plugin` in both binaries and
diff'd against actual registration paths to confirm no other silent dead
plugins remained.

The story is closure paperwork: substantive work landed across four commits
on `main` before the story file was authored. Per the friend-game-lite
orchestrator memory rule (track as evidence, not as a merge gate), the
formal `/story-done` is run retroactively with the audit doc and resolution
commits as evidence.

This story does **not** add new tests, change network protocol, alter
sprint-9 carry-over conditions, claim public release readiness, claim full
playable-client manual QA, claim broad accessibility completion, or close
any S8 / Sprint 9 carried condition.

**Primary sources**:

- `production/qa/evidence/sprint-10-plugin-registration-audit.md` (audit doc — landed at `0648deb`, PROMPT 563)
- `production/sprints/sprint-10.md` (S10-TD-002 row, lines 90-93)
- `production/qa/qa-plan-sprint-10-2026-05-10.md:253-261` (qa-plan checklist)
- `production/session-state/codex-orchestrator-state.md` ("Critical sanity-check pattern — plugin registration audit", 2026-05-09)

**GDD, UX, and TR trace**:

- No GDD requirement. This is a tech-debt audit story — there is no TR-ID
  in `docs/architecture/tr-registry.yaml` for plugin-registration hygiene.
- The audit protects the existing TR-NP / TR-RSM / TR-PAW surface area by
  ensuring the plugins that implement those requirements are actually
  reached by the production binaries.

**ADR Governing Implementation**:

No ADR governs this story directly. ADR-021 (presentation boundaries) and
ADR-011 (network protocol) constrain what each plugin is allowed to do
once registered, but the registration audit itself is plumbing-level
hygiene with no protocol or architecture decision involved.

**Engine**: Bevy 0.18 (Rust) | **Risk**: MEDIUM (silent failure class)

**Engine Notes**: Bevy 0.18's `App::add_plugins(...)` is the only mechanism
that puts a `Plugin::build(&mut self, app: &mut App)` call into the App's
construction path. A plugin that is `pub`-exposed but never reached by an
`.add_plugins(...)` call from any `[[bin]]` entry-point is silently absent
from the running binary — no compile error, no runtime warning, no test
failure unless an E2E test boots the production App. This story does not
add such an E2E test (recommended as separate follow-up tech debt; see
audit doc Phase 4.4).

**Control Manifest Rules (2026-05-05)**: Not applicable — this story
modifies neither presentation, networking, nor gameplay code paths beyond
the single-line plugin registration in `client/src/main.rs` (already
landed at `8932d8c`).

---

## Scope

### In Scope

- Enumerate every `pub struct *Plugin` declared under `server/src/feature/*`,
  `server/src/core/*`, and `client/src/**/*plugin*.rs` (plus equivalent
  paths) in the audit doc.
- Diff against `add_plugins(...)` calls in `server/src/main.rs` and
  `client/src/main.rs` (including transitive registrations via parent
  plugins like `PresentationPlugin` and `ServerNetworkPlugin`).
- Resolve every defined-but-not-registered plugin by one of:
  - registering it in the appropriate `main.rs` (behaviour-change-flagged path);
  - deleting it (dead-code path);
  - annotating it `#[allow(dead_code)]` with a comment pointing at the decision.
- Author the audit doc at `production/qa/evidence/sprint-10-plugin-registration-audit.md`.
- Run the same audit on the client binary path.
- Document the result table per binary (server / client / dedicated harness binaries).
- Capture the panic-prevention cascade from `KeywordPlugin` registration
  (5 latent `todo!()` stubs in `server/src/feature/keyword/observers.rs`)
  via the resolution commit referenced in completion notes.

### Out of Scope

- No new E2E test that boots the production App and asserts every declared
  plugin is in the App's plugin registry. Recommended as separate follow-up
  tech debt per audit doc Phase 4.4.
- No new automated test of any kind (audit doc is the deliverable per
  qa-plan).
- No changes to `server/src/network/`, `client/src/presentation/` rendering
  logic, or any gameplay system beyond the single-line plugin registrations
  and the 5-line panic-prevention conversion.
- No closure of S8-QA-001-W1, QA-COND-0005, or QA-COND-0006.
- No claim of public release readiness, full playable-client manual QA,
  full game completion, or broad Standard-tier accessibility completion.
- No Sprint 10 activation or close-out claims (covered separately).
- No edits to `production/sprints/sprint-10.md` Acceptance Criteria column
  beyond what `/story-done` writes.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-10.md:93` S10-TD-002 row.)

- [x] **Server enumeration**: GIVEN the audit doc, WHEN the server table is
      reviewed, THEN every `pub struct *Plugin` under `server/src/feature/*`
      and `server/src/core/*` is enumerated with file path and source line.
      *Evidence*: audit doc Phase 2 (14-row table at lines 91-106).
- [x] **Server registration diff**: GIVEN the server table, WHEN the
      "How registered" column is read, THEN every plugin maps to an
      `add_plugins(...)` call in `server/src/main.rs:108-136` or to a
      transitive parent (e.g. `EconomyNetworkPlugin` via
      `ServerNetworkPlugin`). *Evidence*: same Phase 2 table.
- [x] **Defined-but-not-registered resolution**: GIVEN any plugin defined
      but not registered, WHEN the resolution path is chosen, THEN the
      plugin is either added to `App` (behaviour-change-flagged) or
      deleted (dead-code) or `#[allow(dead_code)]`-annotated with a
      decision-pointing comment. *Evidence*:
      - `AssetWiringPlugin` registered at `8932d8c` (PROMPT 569) — single
        `add_plugins(AssetWiringPlugin)` line in `client/src/main.rs`.
      - `BoardWasmPerfHarnessPlugin` deleted at `bbdb91e` (PROMPT 570) —
        verbatim duplicate of `BoardRenderingPerfHarnessPlugin`; deletion
        loses no behaviour because the body is identical.
      - Server: 0 unregistered plugins remain (closed at `d7211f1` PROMPT 545).
- [x] **Audit doc location**: GIVEN the deliverable spec, WHEN the audit
      doc path is checked, THEN it exists at
      `production/qa/evidence/sprint-10-plugin-registration-audit.md`.
      *Evidence*: file landed at `0648deb` (PROMPT 563).
- [x] **Client audit**: GIVEN the client binary, WHEN the same audit is
      run on `client/src/main.rs` for `client/src/*/plugin.rs` files, THEN
      every `pub struct *Plugin` is enumerated and matched to its
      registration site. *Evidence*: audit doc Phase 3 (15-row table at
      lines 118-140).
- [x] **Result documented**: GIVEN the client audit result, WHEN the
      classification is read, THEN each defined-but-not-registered client
      plugin has a documented disposition (behaviour-change-flagged,
      deletion-candidate-flagged, or dedicated harness binary).
      *Evidence*: audit doc Phase 5 (lines 199-273).
- [x] **No silent dead-plugin paths remain**: GIVEN both binaries, WHEN the
      registration trees are walked at the post-resolution commit set
      (`0648deb` + `bbdb91e` + `8932d8c`), THEN every defined plugin is
      reached by at least one `[[bin]]` entry-point or has been deleted.
      *Evidence*: server 14/14 reached; client 13/15 reached in main game
      client + 1 reached in `board_rendering_perf_harness` binary;
      remaining 1 (the duplicate `BoardWasmPerfHarnessPlugin`) deleted at
      `bbdb91e`. **Net post-resolution: 14/14 server + 14/14 client = 0
      silent dead plugins.**

---

## Implementation Notes

The substantive work landed across four commits on `main` before this story
file was authored:

1. `0648deb` (PROMPT 563) — pre-stage audit doc; docs-only, no source edits.
2. `bbdb91e` (PROMPT 570) — delete duplicate `BoardWasmPerfHarnessPlugin`
   (`client/src/presentation/board_rendering/perf_harness.rs` -8 lines).
3. `8932d8c` (PROMPT 569) — register `AssetWiringPlugin` in
   `client/src/main.rs` (+2 lines: import + `add_plugins`).
4. `f06271a` (PROMPT 588) — replace 5 `todo!()` stubs in
   `server/src/feature/keyword/observers.rs` with `tracing::warn!` no-ops.
   This is the cascade fix from `d7211f1` (KeywordPlugin registration):
   registering KeywordPlugin made dormant `todo!()` panics fire at
   DRAFT_INITIAL round start. Each stub became a one-line warn-and-return
   no-op documenting that real keyword dispatch is deferred to a future
   story.

The audit doc is the canonical evidence and primary deliverable.

## Performance Budget

N/A — single-line plugin registrations and audit documentation only. No
hot-path code changed. No measurable runtime cost from `add_plugins` calls
at App construction.

---

## QA Test Cases

(Source: `production/qa/qa-plan-sprint-10-2026-05-10.md:253-261`.)

- **Plugin enumeration grep**
  - Given: server and client source trees.
  - When: `grep -rn "pub struct .*Plugin" server/src/` and equivalent
    client grep are run.
  - Then: the audit doc's tables enumerate every match with source line.

- **Registration diff**
  - Given: the audit doc tables.
  - When: each "How registered" cell is checked against `main.rs`.
  - Then: every plugin maps to a real `add_plugins(...)` call or a
    transitive parent's `build()`.

- **Resolution disposition**
  - Given: the audit doc Phase 5 classification.
  - When: each flagged plugin is checked against the post-audit commit set.
  - Then: each is registered, deleted, or annotated per the chosen
    disposition.

---

## Test Evidence

**Story Type**: Config/Data

**Required evidence document**:

- `production/qa/evidence/sprint-10-plugin-registration-audit.md`

**Required source evidence before this story can close**:

- Audit doc lands on `main` (✅ `0648deb`).
- Each flagged client plugin has a chosen disposition landed on `main`
  (✅ `8932d8c` for `AssetWiringPlugin`; ✅ `bbdb91e` for
  `BoardWasmPerfHarnessPlugin`).
- KeywordPlugin registration cascade hardened (✅ `f06271a`).

**Required verification commands**:

- `git log --oneline 0648deb -1` (audit doc commit exists)
- `git log --oneline bbdb91e -1` (deletion commit exists)
- `git log --oneline 8932d8c -1` (AssetWiringPlugin registration commit exists)
- `git log --oneline f06271a -1` (panic-prevention commit exists)
- `git branch --contains <sha>` returns `main` for all four

**Status**: All four commits verified present on `main` at
`/story-done` time (2026-05-10).

---

## Dependencies

- Depends on: `d7211f1` (PROMPT 545 — `CardPoolPlugin` + `KeywordPlugin`
  server registration fix). This was the breakthrough that motivated the
  audit.
- Depends on: Sprint 10 plan (`production/sprints/sprint-10.md`) and Sprint
  10 QA plan (`production/qa/qa-plan-sprint-10-2026-05-10.md`) being
  authored.

## Readiness Notes

**Implementation readiness verdict**: COMPLETE (as of 2026-05-10).

Pull condition was met before this `/story-done` ran:
- Audit doc at `0648deb` exists and is exhaustive.
- Both flagged client plugins resolved on `main` at `bbdb91e` + `8932d8c`.
- Cascade panic-prevention landed at `f06271a`.
- Sprint 10 was activated at `8ff4f84` (PROMPT 591).

---

## Completion Notes

**Completed**: 2026-05-10
**Prompt**: 600
**Criteria**: 7/7 passing (all auto-verified by audit-doc inspection +
git-log verification of resolution commits)
**Verdict**: COMPLETE WITH NOTES

**Resolution commit trail (all on `main`)**:

| Commit | Role | Files | Lines |
|---|---|---|---|
| `0648deb` (PROMPT 563) | Audit doc pre-stage | `production/qa/evidence/sprint-10-plugin-registration-audit.md` | +325 (new file) |
| `bbdb91e` (PROMPT 570) | `BoardWasmPerfHarnessPlugin` deletion | `client/src/presentation/board_rendering/perf_harness.rs` | -8 |
| `8932d8c` (PROMPT 569) | `AssetWiringPlugin` registration | `client/src/main.rs` | +2 |
| `f06271a` (PROMPT 588) | KeywordPlugin observer panic-prevention | `server/src/feature/keyword/observers.rs` | +15 / -5 |

**Deviations**:

- ADVISORY: Story file was authored retroactively (after substantive work
  landed) per the friend-game-lite orchestrator memory rule that treats
  closure paperwork as evidence rather than a merge gate.
- ADVISORY: `f06271a` (5 `todo!()` → `tracing::warn!` no-op) was outside
  the original AC list but logically completes "no silent dead-plugin
  paths" — it removed the latent panic surface that an unregistered
  plugin's stubs had hidden.

**Test Evidence**: Config/Data — audit document at
`production/qa/evidence/sprint-10-plugin-registration-audit.md` exists and
satisfies AC1-7 by direct inspection. No automated test required per qa-plan.

**Code Review**: Skipped — Lean mode; deliverable is an audit document
with three thin resolution commits, each with self-contained verification
in its commit message.

**Carried state preserved**:

- Sprint 9 closed-with-conditions disposition unchanged.
- S8-QA-001-W1 manual/browser two-client GAME_OVER gap remains open.
- QA-COND-0005 (Standard-tier accessibility) remains accepted-risk
  friend-game scope.
- QA-COND-0006 (playtest fun-hypothesis validation) remains
  accepted-risk / deferred.
- No public release readiness, full playable-client manual QA, full game
  completion, or broad accessibility completion is claimed.

**Recommended follow-up tech debt** (out of this story's scope per audit
doc Phase 4.4): expose `build_app(app: &mut App)` from `server/src/main.rs`
and `client/src/main.rs` as a library function and write a single E2E boot
test that asserts every declared `pub struct *Plugin` is present in the
App's plugin registry after `build_app` runs. That test would have caught
the original DRAFT_INITIAL silent failure and would catch any future
`AssetWiringPlugin`-style regression. Belongs as its own story under the
playable-client epic.
