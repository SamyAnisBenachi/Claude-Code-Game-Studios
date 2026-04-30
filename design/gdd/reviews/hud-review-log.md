# Review Log — HUD GDD

## Review — 2026-04-30 — Verdict: MAJOR REVISION NEEDED → Revised in-session
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, ux-designer, network-programmer, audio-director, creative-director
Blocking items: 10 resolved | Recommended: 9 noted
Summary: Structural layout failure (diagonal gold zones were physiologically implausible for peripheral omniscience) resolved by consolidating both gold readouts to top-right. Inter-GDD contradiction (S2CGameSnapshot.PlayerSnapshot missing reserved_gold field, required by NP GDD amendment). Rule 10 vs Rule 13 GAME_OVER reconnect tiebreak defined (snapshot always wins, then re-freeze). Seven new BLOCKING ACs added (round counter format, GAME_OVER visibility, HUD root at LOBBY, cold-start placeholder, ObjectiveDestroyed idempotency, FROZEN+snapshot, sub-label entity count). Four AC field name specification errors corrected. OQ-HUD-01 reclassified as gameplay correctness blocker (NP GDD must define S2CSessionPaused before timer-bearing phases can ship); OQ-HUD-02 design-rejected (settings flag recreates screen-share leak).
Prior verdict resolved: No — first review
