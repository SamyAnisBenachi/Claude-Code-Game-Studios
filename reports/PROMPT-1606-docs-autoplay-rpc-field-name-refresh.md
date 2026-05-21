# PROMPT 1606 — DOCS-AUTOPLAY-RPC-FIELD-NAME-REFRESH

**Branch**: `main`
**Commit (HEAD)**: `237572aff25e240bcc9a878d10710c20329c0db5` (origin/main, pre-edit)
**Date (UTC)**: 2026-05-21
**Lane**: doc-only refresh. No code, no Cargo.

---

## Verdict

`1606: DOCS-AUTOPLAY-RPC-FIELD-NAME-REFRESH: SHIPPED`

Closes the doc-only drift surfaced by PROMPT 1605
(`reports/PROMPT-1605-post-1601-autoplay-focused-verify.md`) between
`docs/autoplay.md` and the Rust-canonical autoplay RPC schema in
`client/src/autoplay.rs`. The skill (`skills/ccgs-autoplay/SKILL.md`) and
the tool README (`tools/autoplay/README.md`) were already correct and are
not touched.

---

## Files touched

| Path | Change |
| --- | --- |
| `docs/autoplay.md` | RPC field-name + launcher path refresh (5 hunks, doc-only) |
| `reports/PROMPT-1606-docs-autoplay-rpc-field-name-refresh.md` | this report |

Zero source files modified. Zero Cargo files modified. No
`production/session-state/**`, no sprint/status files, no QA evidence, no
tools source.

Pre-existing unrelated `M .claude/settings.json` in working tree was left
alone (not in this prompt's scope; surfaced in 1605 as unrelated).

## Concrete edits in `docs/autoplay.md`

1. **§"RPC surface (v1, additive forever)"** — table row updates:
   - `autoplay/capabilities` result: `{ version, methods, env: {…} }` →
     `{ version: u32, methods: { … }, input: {…}, invariants: string }`.
   - `autoplay/input` params: `keys?`, `mouse_buttons?`, `scroll?: { x, y }` →
     `keys_down?`, `keys_up?`, `mouse_down?`, `mouse_up?`,
     `scroll?: [f32, f32]`.
   - `autoplay/screenshot` result: `path: string` → `relative_path: string`.
2. **§"Architecture" ASCII artifact tree** (line 115) — launcher rename
   `run-autoplay.ps1` → `Run-AutoplaySmoke.ps1`.
3. **§"Interop with existing repo infrastructure" → `tools/dev-launcher-app/`
   row** (line 202) — launcher rename `run-autoplay.ps1` →
   `Run-AutoplaySmoke.ps1`.
4. **§"Verification policy"** (line 211) — launcher rename `run-smoke.ps1`
   → `Run-AutoplaySmoke.ps1`.

These match the Rust source-of-truth in `client/src/autoplay.rs`:

- `capabilities_json()` (line ~608) emits keys `version`, `methods`,
  `input`, `invariants` — no `env`.
- `parse_input()` (line ~666) reads `keys_down` / `keys_up` / `mouse_down`
  / `mouse_up` arrays and `scroll` as a 2-element array `[x, y]`.
- Screenshot result emits `relative_path` (line ~586,
  `"{{\"queued\":{seq},\"relative_path\":{rel}}}"`).
- Launcher file on disk: `tools/autoplay/Run-AutoplaySmoke.ps1` (confirmed
  via `ls`).

## Validation

- **Path allowlist review**: changed paths confined to `docs/autoplay.md`
  (owned-scope) + this report file (owned-scope). No source, Cargo,
  session-state, sprint, QA-evidence, or tools-source files touched.
- **`git diff --check docs/autoplay.md`**: clean (no whitespace errors, no
  conflict markers).
- **Stale-name sweep** in `docs/autoplay.md` post-edit:
  - `run-smoke.ps1` → 0 hits.
  - `run-autoplay.ps1` → 0 hits.
  - `keys?` (with question-mark, in old input-param sense) → 0 hits.
  - `mouse_buttons?` → 0 hits.
  - screenshot `path:` (in result-object sense) → only `relative_path:` now;
    the bare `path` field name is gone.
  - `capabilities … env:` → 0 hits.
- **No Cargo run**, **no broad validation**, per task directive. Cargo
  feature surface and Rust code untouched.

## Commands actually run

```
git rev-parse HEAD                         # 237572af…
git status --porcelain                     # only pre-existing .claude/settings.json
Read docs/autoplay.md                      # full
Read reports/PROMPT-1605-post-1601-autoplay-focused-verify.md
Read client/src/autoplay.rs (header + capabilities_json + parse_input)
ls tools/autoplay/                         # confirms Run-AutoplaySmoke.ps1
Grep "run-smoke|run-autoplay|keys\\?|mouse_buttons\\?" docs/autoplay.md   # all 0 hits post-edit
git --no-pager diff --cached docs/autoplay.md                               # 5 hunks, all expected
git diff --check docs/autoplay.md                                          # clean
```

---

## Final line

1606: DOCS-AUTOPLAY-RPC-FIELD-NAME-REFRESH: SHIPPED
