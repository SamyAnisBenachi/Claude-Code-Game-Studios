# Review Log — Card Animations

## Review — 2026-04-30 (Pass 2) — Verdict: NEEDS REVISION
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, gameplay-programmer, performance-analyst, audio-director, ux-designer, network-programmer, creative-director (senior synthesis)
Blocking items: 24 new | Recommended: 15
Summary: Re-review after Pass 1's 13 blockers were resolved in-session. Pass 2 surfaced 24 new blockers across 6 specialist domains. The architecture (domain-event indirection, AnimGroup/AnimQueue, Decoration Test philosophy) remains sound. Critical gaps: OQ-CA-06 was mislabeled "Resolved" while pointing to the wrong Bevy 0.18 API (`add_event` does not exist in 0.17+; correct: `add_message`/`MessageReader`); `PlacementCancelAllAnimsRequested` was missing from the domain event table (CA-21 had no delivery mechanism); F2 tuning table had three mathematical errors; `impact_flash_audio_offset_ms` was a phantom config key with a wrong derivation (67 ms at 60 fps; corrected to ~17 ms). All 24 blockers resolved in-session. 12 OQs remain open (5 Bevy API verification, 3 cross-system recommendations, 4 new). Re-review in fresh session recommended.
Prior verdict resolved: Yes — Pass 1 (2026-04-30) NEEDS REVISION resolved

## Review — 2026-04-30 — Verdict: NEEDS REVISION
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, gameplay-programmer, network-programmer, performance-analyst, ux-designer, audio-director, creative-director (senior synthesis)
Blocking items: 13 | Recommended: 10
Summary: The GDD's core architecture (domain-event indirection, restrained animation philosophy, decoration test) is conceptually sound. Critical issues were: `add_message` is not Bevy 0.18 API (should be `add_event`); `sprite.color.set_alpha()` unverified (OQ-CA-10 added); DRAFT_INITIAL violated Rule C-14 (fixed via card-draw sequencing at t+350ms); audio timing contract had no delivery mechanism (fixed via offset-based model); PLACEMENT→RESOLUTION normal-path transition was unspecified (force-cancel edge case + CA-21 added); F1 formula had wrong i-range (0–4→0–3); F2 constraint allowed silent violation at max tuning values. All 13 blockers resolved in-session. 10 recommended items addressed. Re-review in fresh session recommended.
Prior verdict resolved: No — first review
