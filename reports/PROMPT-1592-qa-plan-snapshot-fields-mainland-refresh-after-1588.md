# PROMPT 1592 — QA-PLAN-SNAPSHOT-FIELDS-MAINLAND-REFRESH-AFTER-1588

## Outcome

**STATUS: REFRESHED**

PROMPT 1590's payload (1586 QA-snapshot resolution-phase fields + 1587 Sprint 18
QA-plan paperwork) was refreshed onto current `origin/main` after PROMPT 1588 bot
Wave 2 landed. The new integration branch is strict-FF-ready vs `origin/main`.

## Refresh source-of-truth

| Field | Value |
|---|---|
| Stale base (1590 was FF-ready against) | `origin/main@9be8827f` |
| New base (current) | `origin/main@945cbd71fe64de6d1c1dc078c3fc358c3903a08b` |
| Source integration branch | `origin/integrate/qa-plan-snapshot-fields-1590@bc8ae58f` |
| New integration branch | `integrate/qa-plan-snapshot-fields-1592` |
| New branch HEAD | `3f583781170af76748e1e063b2d4f355f263a55a` |
| Worktree | `D:/Tmp/wt-1592` |

## FF-readiness verification

```
$ git merge-base --is-ancestor origin/main HEAD && echo OK
OK
$ git log --oneline origin/main..HEAD
3f583781 PROMPT-1590 integration refresh report for QA-plan paperwork + QA-snapshot resolution-phase fields (1586+1587 onto origin/main 9be8827f)
2c95da34 qa-plan-paperwork(s18): discharge sprint-18 §0.4 qa-plan-found stale-false correction (PROMPT 1587)
5294095e PROMPT-1586 QA snapshot resolution-phase observability fields
```

`origin/main@945cbd71` is a proper ancestor of `integrate/qa-plan-snapshot-fields-1592@3f583781`.
Strict FF land is unblocked.

## Refresh method

1. Created dedicated worktree `D:/Tmp/wt-1592` with new branch
   `integrate/qa-plan-snapshot-fields-1592` checked out from current
   `origin/main@945cbd71`.
2. Cherry-picked the three carry commits in original order:
   - `fbc3e990` → `5294095e` (PROMPT 1586 QA snapshot resolution-phase fields)
   - `d5dfa3a3` → `2c95da34` (PROMPT 1587 QA-plan paperwork)
   - `bc8ae58f` → `3f583781` (PROMPT 1590 integration refresh report)
3. All three cherry-picks applied cleanly with **zero conflicts** (no overlap
   with the 1582/1583/1588 bot Wave 2 file set).
4. No additional edits applied. Payloads preserved byte-for-byte modulo
   cherry-pick author/commit metadata.

## Path allowlist review

| File | Allowlist disposition |
|---|---|
| `client/src/presentation/qa_snapshot.rs` | ✅ explicitly owned by this PROMPT |
| `tests/integration/qa_snapshot/resolution_phase_field_coverage_test.rs` | ✅ explicitly owned ("tests/integration/qa_snapshot/** changed by 1586") |
| `production/sprint-status.yaml` | ✅ allowed ("only if changed by 1587 for QA-plan disposition row" — confirmed: §0.4 qa-plan-found stale-false correction) |
| `reports/PROMPT-1586-qa-snapshot-resolution-phase-fields-followup.md` | ✅ carry-over from 1586 (allowed by "Include/carry reports from 1586/1587/1590") |
| `reports/PROMPT-1590-qa-plan-snapshot-fields-integration-refresh.md` | ✅ carry-over from 1590 |
| `client/Cargo.toml` | ⚠️ noted below — intrinsic to 1586's payload |

### Note on `client/Cargo.toml`

The 1586 commit (`fbc3e990`) touches `client/Cargo.toml` to add a single
`[[test]]` registration for the new
`qa_snapshot_resolution_phase_field_coverage_test` (following the established
PROMPT 1186 / 1229 / 1347 test-registration pattern). Without this entry, the
new integration test cannot be discovered or executed by Cargo. The change is
strictly scoped to a test-discovery entry; it adds no dependencies, no
profile-edits, no toolchain change. It is mechanically required to preserve
PROMPT 1586 functionally, so it is carried forward with the same scope-note
discipline as the original 1586 ship. Surfaced here for orchestrator
disposition awareness; not a scope expansion beyond the source 1586 payload.

`reports/PROMPT-1592-…md` (this file) is owned by this PROMPT per task scope.

## `git diff --check` (whitespace / conflict markers)

```
$ git diff --check origin/main..HEAD
(no output — clean)
```

## Forbidden-path audit

Confirmed **no edits** to the explicitly forbidden set:
- `client/src/ui/hand/mod.rs` — untouched
- `client/src/presentation/board_rendering.rs` — untouched
- `client/src/ui/shop_auction/mod.rs` — untouched
- `server/**` — untouched
- `shared/**` — untouched
- `production/session-state/**` — untouched
- `production/sprints/**` — untouched
- `production/stage.txt` — untouched
- Other unrelated Cargo / CI files — untouched (only `client/Cargo.toml`
  carries the 1586 test-registration line as documented above)

No other workers' edits were reverted; the cherry-picks layered cleanly on top
of 1582/1583/1588's bot Wave 2 changes.

## Paperwork non-claim

This refresh introduces **no** Sprint 18 close-out, **no** Sprint 19 activation,
**no** stage change, and **no** release-readiness claim. Paperwork content is
limited to the same §0.4 `qa-plan-found` stale-false correction that PROMPT
1587 already shipped against `origin/main@9be8827f` — re-carried verbatim onto
`origin/main@945cbd71` with no semantic change.

## Validation scope

Per task instructions ("focused QA snapshot tests only if cheap; broad
verification deferred to VERIFY lane"): the cherry-picks were content-identical
re-applications of commits previously shipped on PROMPT 1590, so no fresh
compile/test was executed in this refresh worker. Broad Cargo build / lane
verification is **deferred to the next VERIFY lane** for the
`integrate/qa-plan-snapshot-fields-1592` branch.

## Push disposition

`integrate/qa-plan-snapshot-fields-1592` exists locally at
`3f583781170af76748e1e063b2d4f355f263a55a` in worktree `D:/Tmp/wt-1592`.
Push attempt to `origin` is recommended next step (only the integration branch;
`main` is not touched). Reporting branch + commit so the orchestrator can
schedule the push and MAINLAND_ENQUEUE lane against current `origin/main`.

If the push lane is blocked (protected-branch / GitHub export / network), the
exact local refs to relay are:

- branch: `integrate/qa-plan-snapshot-fields-1592`
- commit: `3f583781170af76748e1e063b2d4f355f263a55a`
- worktree path: `D:/Tmp/wt-1592`
- recommended push command: `git push -u origin integrate/qa-plan-snapshot-fields-1592`

---

1592: QA-PLAN-SNAPSHOT-FIELDS-MAINLAND-REFRESH-AFTER-1588: REFRESHED
