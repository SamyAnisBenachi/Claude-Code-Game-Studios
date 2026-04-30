# Review Log — Card Animations

## Review — 2026-04-30 — Verdict: NEEDS REVISION
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, gameplay-programmer, network-programmer, performance-analyst, ux-designer, audio-director, creative-director (senior synthesis)
Blocking items: 13 | Recommended: 10
Summary: The GDD's core architecture (domain-event indirection, restrained animation philosophy, decoration test) is conceptually sound. Critical issues were: `add_message` is not Bevy 0.18 API (should be `add_event`); `sprite.color.set_alpha()` unverified (OQ-CA-10 added); DRAFT_INITIAL violated Rule C-14 (fixed via card-draw sequencing at t+350ms); audio timing contract had no delivery mechanism (fixed via offset-based model); PLACEMENT→RESOLUTION normal-path transition was unspecified (force-cancel edge case + CA-21 added); F1 formula had wrong i-range (0–4→0–3); F2 constraint allowed silent violation at max tuning values. All 13 blockers resolved in-session. 10 recommended items addressed. Re-review in fresh session recommended.
Prior verdict resolved: No — first review
