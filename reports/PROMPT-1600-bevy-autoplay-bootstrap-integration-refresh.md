# PROMPT 1600 — BEVY-AUTOPLAY-BOOTSTRAP-INTEGRATION-REFRESH

## Scope
Refresh and integrate PROMPT 1595's autoplay bootstrap first slice
(`work/bevy-autoplay-bootstrap-1595 @ 14f1536f`) onto current
`origin/main @ b3dc0a39` (post-PROMPT 1599 bot-flow Wave A/B/C landing).

## Source/Target
- Source commit: `14f1536f` (PROMPT 1595 BEVY-AUTOPLAY-BOOTSTRAP-FIRST-SLICE)
  parented at `3a4603af` (pre-PROMPT 1596).
- Target tip: `origin/main @ b3dc0a39`
  (`PROMPT-1599 report: bot-flow MVP Wave A/B/C integration refresh`).
- Drift between 1595 parent and current main: 4 commits (PROMPT 1596 lobby
  add/remove bot UX, 1597 server QA snapshot/decision-log, 1598 auction bid
  funnel Wave 2.5, 1599 integration refresh report).

## Result
- Worktree: `D:/Tmp/wt-1600`
- Branch: `integrate/bevy-autoplay-bootstrap-1600`
- Integration tip: `d69a2a81a3e531001a02571b7c4f873557a508c9`
- Parent: `b3dc0a39` (`origin/main`) — branch is strict-FF-ready vs
  `origin/main` (verified `git merge-base --is-ancestor origin/main HEAD` true).

## Cherry-pick outcome
`git cherry-pick 14f1536f` succeeded with one auto-merge:
- `client/Cargo.toml` — PROMPT 1595 added the `autoplay-remote` feature stanza;
  PROMPTs 1596–1599 added/edited unrelated stanzas in the same file. Git's
  3-way merge interleaved the additions cleanly, no manual resolution required.

All other PROMPT 1595 files are net-new and conflict-free:
- `client/src/autoplay.rs` (new, 1264 LOC)
- `client/src/lib.rs` (1 cfg-gated `pub mod autoplay;` insertion preserved)
- `client/src/main.rs` (1 cfg-gated `app.add_plugins(client::autoplay::AutoplayPlugin)` preserved)
- `docs/autoplay.md` (new, architecture spec + hard invariants)
- `reports/PROMPT-1595-bevy-autoplay-bootstrap-first-slice.md` (new)
- `skills/ccgs-autoplay/SKILL.md` (new, project-local runbook)
- `tools/autoplay/README.md`, `Run-AutoplaySmoke.ps1`, `driver.py`, `rpc.py` (all new)

`git diff --check HEAD~1 HEAD` — clean (no whitespace/conflict artefacts).

## Owned-scope compliance
All 11 modified paths fall within PROMPT 1600's owned scope:
- `docs/autoplay.md` ✓
- `skills/ccgs-autoplay/SKILL.md` ✓
- `tools/autoplay/**` ✓
- `client/src/autoplay.rs`, `client/src/lib.rs`, `client/src/main.rs` ✓ (dev-only harness wiring)
- `client/Cargo.toml` ✓ (autoplay-remote feature only — PROMPT 1595's stanza)
- `reports/PROMPT-1595-...md` ✓ (carried; not yet on main)
- `reports/PROMPT-1600-...md` ✓ (this report)

No edits to `production/**`, `Cargo.lock`, server, shared, gameplay logic,
sprint paperwork, session state, or CI files.

## PROMPT 1599 / autoplay interaction
No semantic interaction between the two patches:
- PROMPT 1599 bot-flow touches `client/src/ui/lobby.rs`, `server/**`,
  `tests/**`, and adds server-side bot QA snapshot/decision-log + auction bid
  funnel Wave 2.5. None of those surfaces overlap with the autoplay BRP plugin
  or the external Python driver.
- The only shared file is `client/Cargo.toml`, where PROMPT 1599 added bot
  control deps and PROMPT 1595 added the `autoplay-remote` feature stanza.
  Git merged the two additions cleanly; the resulting feature set is the
  union (`default = ["ui_picking"]`, `ui_picking`, `wgpu-validation`,
  `autoplay-remote`) with no duplication.

Autoplay invariants preserved: low-level input only
(`ButtonInput<KeyCode>/<MouseButton>`, `Window` cursor warp,
`MouseWheel`), dev-only via Cargo feature `autoplay-remote` + runtime
`CCGS_AUTOPLAY=1`, JSON-RPC bound to `127.0.0.1:15873`, no semantic
gameplay mutation endpoints exposed.

## Validation
- `git diff --check HEAD~1 HEAD` — clean.
- Path allowlist — all 11 paths in owned scope.
- Focused build: `cargo check -p client --features autoplay-remote`
  - Finished `dev` profile in 0.45s.
  - 101 pre-existing deprecation warnings (12 duplicates) on
    `ShopAuctionUiEntity` — all PRE-EXISTING, identical to PROMPT 1595's
    report ("101 pre-existing deprecation warnings unrelated; zero new
    warnings from autoplay.rs").
  - No new warnings or errors introduced by the refresh.
- Broad cargo / test suite deferred to VERIFY lane per PROMPT 1600 rules.

## FF-readiness
`git merge-base --is-ancestor origin/main HEAD` → true.
Branch `integrate/bevy-autoplay-bootstrap-1600` (tip `d69a2a81`) is
strict-FF-ready against `origin/main @ b3dc0a39`.

## Push
Push attempt and outcome appended below this line at completion.

1600: BEVY-AUTOPLAY-BOOTSTRAP-INTEGRATION-REFRESH: SHIPPED
