# PROMPT 1806 — Win32 Capture Fallback Mainland Refresh After 1804

## Task
Refreshed integration branch applying 1805 payload on top of 1804 stale-pyc main.

## Branch
`prompt-1806-win32-capture-fallback-mainland-refresh` based on `origin/main@cf471c75`

## Cherry-Picks Applied
- 6369d0bf feat(autoplay): PROMPT 1803 — Win32 capture fallback hardening
- 6bb9b9c2 docs(reports): PROMPT 1803 — Win32 capture fallback hardening report
- 902e9519 docs(reports): PROMPT 1805 — Win32 capture fallback integration refresh report

All three cherry-picks applied cleanly with no conflicts. The 1805 report had
trailing whitespace on lines 3-5 (Markdown line-break spaces) which was stripped
before finalising the amended commit.

## Validation
- FF-eligible: PASS
- diff --check: PASS (trailing whitespace in 1805 report stripped before amend)
- No 1802/1804 deletes: PASS (only A/M entries in diff --name-status)
- pytest win32: PASS — 37 tests passed
- pytest stale-pyc: PASS — 5 tests passed
- Total: 42/42 passed in 0.14s

## Files Added by This Branch
- `reports/PROMPT-1803-autoplay-window-foreground-win32-capture-fallback-repair.md` (new)
- `reports/PROMPT-1805-autoplay-win32-capture-fallback-integration-refresh.md` (new)
- `tools/autoplay/win_capture.py` (modified — Win32 capture hardening)
- `tools/autoplay/driver.py` (modified — Win32 capture wiring)
- `tests/tools/autoplay/test_win32_capture.py` (modified — expanded test suite)
- `reports/PROMPT-1806-autoplay-win32-capture-fallback-mainland-refresh-after-1804.md` (this file)

## Status
SHIPPED

1806: AUTOPLAY-WIN32-CAPTURE-FALLBACK-MAINLAND-REFRESH-AFTER-1804: SHIPPED
