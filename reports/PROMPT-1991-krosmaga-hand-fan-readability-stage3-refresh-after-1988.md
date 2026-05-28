# PROMPT 1991 — Krosmaga Hand Fan Readability Stage3 Refresh After 1988

## Summary

Reapplied the PROMPT 1981 hand fan readability Stage3-D payload onto a fresh
base at `origin/main@32ca23e8` (PROMPT 1988). The stale `origin/work/PROMPT-1981`
branch (commit `957ac14e`) was NOT_FF relative to PROMPT 1988 and would have
deleted the bot/autoplay readiness report chain (1935/1970/1985) and the
Krosmaga tier-border report chain (1933/1961/1974/1986/1988). This refresh
performs a clean file-level transplant with zero stale deletes.

## Base

- `origin/main@32ca23e8` — PROMPT 1988 (latest main)

## Source Material

- `origin/work/PROMPT-1981` commit `957ac14e` — Stage3-D payload

## Files Applied

| Path | Action |
|------|--------|
| `client/src/ui/hand/mod.rs` | M — fan_base_margin_px 100→150, fan_half_spread_px 280→380, arc_height_px 20→30 |
| `tests/integration/hand-ui/draft_initial_grid_test.rs` | M — qa_metrics fan_base_y updated 160→110 to match new margin |
| `tests/unit/hand-ui/fan_layout_formula_test.rs` | M — added STAGE3-D 10-card readability invariants test |
| `reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md` | A |
| `reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md` | A |
| `reports/PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md` | A |
| `reports/PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md` | A |
| `reports/PROMPT-1955-krosmaga-hand-fan-readability-stage3-refresh-after-1920.md` | A |
| `reports/PROMPT-1963-krosmaga-hand-fan-readability-stage3-refresh-after-1957.md` | A |
| `reports/PROMPT-1981-krosmaga-hand-fan-readability-stage3-refresh-after-1976.md` | A |
| `reports/PROMPT-1991-krosmaga-hand-fan-readability-stage3-refresh-after-1988.md` | A (this file) |

## Validation

### git diff --name-status origin/main..HEAD

All entries are A or M. Zero D lines. All paths are within owned scope.

### git diff --check origin/main..HEAD

PASS — no trailing whitespace or whitespace errors.

### Preserved Report Chains

- `reports/PROMPT-1933-krosmaga-auction-tier-border-asset-binding-refresh-after-1929.md` ✓
- `reports/PROMPT-1935-bot-autoplay-story-readiness-report-refresh-after-1931.md` ✓
- `reports/PROMPT-1961-krosmaga-auction-tier-border-1933-report-backfill-after-1957.md` ✓
- `reports/PROMPT-1970-bot-autoplay-story-readiness-report-refresh-after-1959.md` ✓
- `reports/PROMPT-1974-krosmaga-auction-tier-border-1933-report-backfill-after-1972.md` ✓
- `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md` ✓
- `reports/PROMPT-1986-krosmaga-auction-tier-border-1933-report-backfill-after-1976.md` ✓
- `reports/PROMPT-1988-krosmaga-auction-tier-border-1933-report-backfill-after-1985.md` ✓

All forbidden report chains intact.

## Bevy 0.18 Compliance

Source changes are pure data/config parameter tuning (`Default` impl values) and
test additions using `World`-based ECS patterns. No deprecated Bundle, EventReader,
EventWriter, or pre-0.15 APIs introduced.

## Branch

`work/PROMPT-1991` — strict-FF from `origin/main@32ca23e8`.

1991: KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1988: READY_FOR_MAINLAND_ENQUEUE
