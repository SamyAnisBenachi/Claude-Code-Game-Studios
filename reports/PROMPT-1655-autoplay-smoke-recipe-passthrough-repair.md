# PROMPT 1655 — AUTOPLAY-SMOKE-RECIPE-PASSTHROUGH-REPAIR

**Date:** 2026-05-27
**Source-of-truth at launch:** origin/main@1b1f2351
**Worktree:** tmpwt-1655-recipe-passthrough (branch: prompt-1655-recipe-passthrough)
**Scope:** Fix `-Recipe` passthrough defect in `Run-AutoplaySmoke.ps1`; update README recipe table.

---

## Defect Addressed

**HIGH — `-Recipe` not forwarded from `Start-AutoplayVsBot.ps1` → `Run-AutoplaySmoke.ps1`**

`Start-AutoplayVsBot.ps1` (PROMPT 1644) passes `-Recipe $Recipe` when calling
`Run-AutoplaySmoke.ps1`, but `Run-AutoplaySmoke.ps1` had no `-Recipe` parameter
and hardcoded `"--recipe", "smoke"` in the driver launch arguments. Any non-default
recipe run via the composite harness raised `ParameterBindingException` and exited
non-zero.

---

## Changes Made

### `tools/autoplay/Run-AutoplaySmoke.ps1`

1. **Added `-Recipe` parameter** (default `"smoke"`) to the `param()` block — backward compatible.
2. **Threaded `$Recipe` into the Python driver call** — replaced the hardcoded `"--recipe", "smoke"` with `"--recipe", $Recipe`.
3. **Updated status progress line** — changed `recipe=smoke` to `recipe=$Recipe` in the `Write-Host` output at RPC-port-bound.
4. **Updated usage comment** — added `-Recipe full-game` example to the header comment block.

### `tools/autoplay/README.md`

Updated the `## Recipes` section (previously stale at PROMPT 1609):
- Added 4 missing recipes: `add-bot-lobby`, `resolution-observe`, `game-over-observe`, `round-loop`.
- Fixed `full-game` checkpoint list: replaced stale `full-game-resolution` with the correct current checkpoints (`full-game-post-placement`, `full-game-post-resolution`, `full-game-complete`).
- Updated section header to reflect PROMPT 1634/1636/1639 additions.

---

## Validation

### PowerShell AST Parse

```
ParseErrors: 0
TokenCount:  659
Params: Port, ArtifactDir, Recipe, Python, DriverTicks, DriverHz, ClientStartupSecs
```

`Recipe` parameter now present; zero parse errors.

### git diff --check

PASS — no whitespace errors.

### Path Allowlist

All edits confined to:
- `tools/autoplay/Run-AutoplaySmoke.ps1` ✓
- `tools/autoplay/README.md` ✓
- `reports/PROMPT-1655-autoplay-smoke-recipe-passthrough-repair.md` ✓

No Rust source, Cargo files, sprint/session-state, or unrelated launcher code touched.

---

## Files Modified

| File | Change |
|---|---|
| `tools/autoplay/Run-AutoplaySmoke.ps1` | Add `-Recipe` param; thread into driver call; update status text |
| `tools/autoplay/README.md` | Add 4 missing recipes; fix `full-game` checkpoint names |

---

1655: AUTOPLAY-SMOKE-RECIPE-PASSTHROUGH-REPAIR: SHIPPED
