# PROMPT 1765 — BOT-DEBUG-OVERLAY-AC5-RULING-DECISION-PACK

Date: 2026-05-28
Status: SHIPPED
`origin/main` SHA at inspection: `7ca41fc49dbfc8f0175b87818860c0c5bebce718`
Prior ruling prep: PROMPT 1741 (2026-05-28, SHIPPED)

---

## Purpose

Produce a concise decision pack for the human AC5 ruling on story
`BOT-DEBUG-OVERLAY-001`. No source, story, sprint, session-state, docs,
stage, or CI files were modified by this prompt.

---

## 1. Current Implemented Behavior

The overlay module (`client/src/presentation/debug_bot_overlay.rs`) is
**compiled into both debug and release binaries**. There is no Cargo
feature flag and no `#[cfg(feature = ...)]` gate.

The plugin is registered unconditionally in `client/src/presentation/mod.rs`:

```rust
app.add_plugins(DebugBotOverlayPlugin);
```

At runtime, two guards determine whether anything actually activates:

| Guard | Where | What it does |
|-------|-------|--------------|
| `CCGS_DEBUG_UI=1` | Client OS env | Enables UI spawn; unset = falls back to `cfg!(debug_assertions)` |
| `CCGS_BOT_DEBUG_UI=1` | Server OS env | Enables server push messages |

**WASM primary target**: `std::env::var` always returns `Err` in a browser
WASM context (no OS environment). The fallback is therefore always
`cfg!(debug_assertions)` = `false` in release WASM builds. A production
player cannot activate the overlay — the code ships in the WASM bundle as
unreachable dead weight (no UI spawned, no per-frame systems running).

**Native build**: Operator-controlled via env var. Unset in a production
launcher → overlay never mounts.

---

## 2. What AC5 Originally Asked For

> **AC5 (story verbatim):** Overlay is not built into release builds
> (`cfg(debug_assertions)`, feature flag, or equivalent compile-time exclusion).

The story specified that the overlay should not be **compiled** into release
binaries, citing `cfg(debug_assertions)`, a feature flag, or equivalent
compile-time mechanism.

**Gap**: The implementation uses runtime env-gating, not compile-time
exclusion. The overlay IS compiled into release binaries; it is absent at
runtime without the env var.

PROMPT 1623 auditor previously called this a PASS on dual-gate safety
grounds. The story author and PROMPT 1670 deferred the final ruling to a
human.

---

## 3. Options A / B / C

### Option A — Accept runtime env-gating as satisfying AC5 (reinterpret)

**What it means**: Declare the story phrase "compile-time exclusion"
interpreted broadly to include strong runtime env-gating. No edits anywhere.

| Dimension | Detail |
|-----------|--------|
| Code change | **None** |
| Story edit | **None** |
| Story-done path | **Immediate** after ruling |
| WASM release safety | Equivalent (env unreachable in browser) |
| Native release safety | Operator-controlled via env var |
| WASM bundle size | Overlay code present (dead, marginal) |
| Maintenance cost | **Lowest** |
| Spec accuracy | Story text and implementation diverge (unedited) — future readers see "compile-time" but implementation is runtime |

**Risk**: Misleading story text. Future maintainers reading "compile-time
exclusion" may not know the implementation chose runtime gating.

---

### Option B — Update AC5 wording to match implementation, then pass

**What it means**: Edit one line in `story-005-bot-debug-overlay.md` so the
AC text reflects the shipped design. Mark AC5 PASS after the edit is committed.

Edit: replace
```
`cfg(debug_assertions)`, feature flag, or equivalent compile-time exclusion
```
with
```
`CCGS_DEBUG_UI=1` env-gating or equivalent runtime exclusion that prevents display in operator-facing builds
```

| Dimension | Detail |
|-----------|--------|
| Code change | **None** |
| Story edit | **Yes — 1 line in AC5 text** |
| Story-done path | After story edit is committed |
| WASM release safety | Same as A (env unreachable in browser) |
| Native release safety | Same as A (operator-controlled) |
| WASM bundle size | Same as A (dead code present) |
| Maintenance cost | **Low** (one wording edit) |
| Spec accuracy | **Story text matches shipped design** |

**Risk**: None material. The safety verdict is identical to Option A;
the only difference is the story text is corrected.

---

### Option C — Require true compile-time exclusion before story-done

**What it means**: AC5 remains BLOCKED. A follow-up PROMPT must add a
Cargo feature flag (e.g. `debug_overlay`) and gate `debug_bot_overlay.rs`
behind `#[cfg(feature = "debug_overlay")]`. Story-done is deferred.

Follow-up work scope: `Cargo.toml` (workspace + client), `mod.rs`,
`debug_bot_overlay.rs` source guards, CI build matrix, release checklist,
architecture doc update.

| Dimension | Detail |
|-----------|--------|
| Code change | **Yes — multiple files, Cargo feature surface** |
| Story edit | None (AC stays blocked) |
| Story-done path | After follow-up PROMPT lands on `origin/main` |
| WASM release safety | **Strict** — overlay absent from WASM bundle entirely |
| Native release safety | **Strict** — overlay not compiled; no accidental activation path |
| WASM bundle size | Measurable saving (dead code removed) |
| Maintenance cost | **Highest** — feature flag must be maintained in CI, release checklist, docs |

**Risk**: Non-trivial churn for an internal debug tool with no user-facing
surface. The concrete improvement (WASM bundle size reduction) is marginal;
the CI/checklist maintenance burden is ongoing.

---

## 4. Recommendation

**Option B.**

1. **WASM safety is already compile-time equivalent.** The browser offers no
   env var surface. The overlay is unreachable in production WASM builds
   regardless of the compile-time gate.

2. **Option A leaves misleading story text.** Future maintainers reading
   "compile-time exclusion" will not know the design diverged. Option B
   corrects the record at near-zero cost.

3. **Option C overhead is disproportionate.** A Cargo feature flag adds
   ongoing CI matrix, release checklist, and doc surface for an *internal*
   debug tool. The safety gain over Option B is marginal (native binary
   size; no impact on WASM security surface).

4. **Validated by prior auditor.** PROMPT 1623 already found the dual-gate
   acceptable. Option B is that same verdict with the story text corrected —
   not a new decision.

---

## 5. Follow-Up Prompts by Choice

### If you choose Option A

```
PROMPT NNNN -- BOT-DEBUG-OVERLAY-001-STORY-DONE-OPTION-A

Run /story-readiness BOT-DEBUG-OVERLAY-001.
When evaluating AC5, apply the following ruling verbatim:

  AC5 is satisfied. The story phrase "compile-time exclusion" is interpreted
  broadly to include strong runtime env-gating. The dual guard
  (CCGS_DEBUG_UI=1 env var checked at spawn + cfg!(debug_assertions)
  default-off) provides equivalent release safety without a compile-time
  feature flag. No story edit or code change required. Mark AC5 PASS.

All other ACs are already PASS per PROMPT 1670 AC status table.
After /story-readiness clears, run /story-done BOT-DEBUG-OVERLAY-001.
Commit story + sprint status updates. Push to origin/main.
Final line: NNNN: BOT-DEBUG-OVERLAY-001-STORY-DONE-OPTION-A: SHIPPED
```

---

### If you choose Option B (recommended)

```
PROMPT NNNN -- BOT-DEBUG-OVERLAY-001-STORY-DONE-OPTION-B

Edit production/epics/bot-and-autoplay/story-005-bot-debug-overlay.md:
In the AC5 row, replace the text:
  `cfg(debug_assertions)`, feature flag, or equivalent compile-time exclusion
with:
  `CCGS_DEBUG_UI=1` env-gating or equivalent runtime exclusion that prevents
  display in operator-facing builds

Then run /story-readiness BOT-DEBUG-OVERLAY-001.
Apply AC5 ruling: Mark AC5 PASS (wording now matches implementation).
All other ACs are already PASS per PROMPT 1670 AC status table.
After /story-readiness clears, run /story-done BOT-DEBUG-OVERLAY-001.
Commit story edit + sprint status updates. Push to origin/main.
Final line: NNNN: BOT-DEBUG-OVERLAY-001-STORY-DONE-OPTION-B: SHIPPED
```

---

### If you choose Option C

```
PROMPT NNNN -- BOT-DEBUG-OVERLAY-001-COMPILE-TIME-GATE

Add a Cargo feature flag `debug_overlay` to the client crate.
Gate debug_bot_overlay.rs behind #[cfg(feature = "debug_overlay")].
Update client/Cargo.toml, client/src/presentation/mod.rs, and
docs/architecture/bot-debug-overlay.md to document the feature flag.
Update the CI build matrix so release builds do not include the feature.
Update the release checklist to confirm the flag is absent from release.
Commit + push to origin/main.
After this lands, run /story-readiness BOT-DEBUG-OVERLAY-001 and
apply the AC5 ruling: the overlay is no longer compiled into release
builds. Mark AC5 PASS. Then run /story-done BOT-DEBUG-OVERLAY-001.
Final line: NNNN: BOT-DEBUG-OVERLAY-001-COMPILE-TIME-GATE: SHIPPED
```

---

## Validation Notes

- Inspection performed in shared root checkout of `origin/main@7ca41fc4`.
  No dedicated worktree was required for this read-only report.
- No source, story, sprint, session-state, docs, stage, or CI files were
  modified.
- PROMPT 1741 (2026-05-28) performed the same analysis at
  `origin/main@511b193e`; findings are consistent.
- `origin/main` SHA at this inspection: `7ca41fc49dbfc8f0175b87818860c0c5bebce718`

---

1765: BOT-DEBUG-OVERLAY-AC5-RULING-DECISION-PACK: SHIPPED
