---
name: card-animations QL-STORY-READY review
description: Story-readiness verdicts for Card Animations epic (9 stories). OQ-CA-01 (AnimatorState API) is the dominant blocker — 8 ACs across Stories 002/003/005/008 depend on it. Stories 003/005/007 are formally BLOCKED on unresolved OQs.
type: project
---

Card Animations QL-STORY-READY review (2026-05-01).

Story 001: ADEQUATE — Logic scaffold + 5 lenses. OQ-CA-01/OQ-CA-05 must resolve before implementation but story ACs are testable once they do.
Story 002: GAPS — AnimatorState enum name (OQ-CA-01) must resolve before any test harness is written; ACs are otherwise complete and well-specified. Marked GAPS not INADEQUATE because reformulation path is clear.
Story 003: BLOCKED — OQ-CA-02 (Tracks<T>) unconfirmed; story formally blocked by own status, ACs reviewed PRE-IMPL.
Story 004: ADEQUATE — AnimQueue RESOLUTION drain + GAME_OVER skip path. All ACs are self-contained, Time<Virtual>-driven. No AnimatorState dependency.
Story 005: BLOCKED — PlacementRevealAnimReady payload and LaneCell component undefined. ACs reviewed PRE-IMPL.
Story 006: ADEQUATE — F1 stagger formula. Three BLOCKINGs all Time<Virtual>-driven and precisely specified.
Story 007: BLOCKED — OQ-CA-11 (jitter table), DespawnAfter component, text entity layout all undefined. ACs for CA-8/CA-9 reviewed PRE-IMPL (adequate once DespawnAfter is defined).
Story 008: GAPS — CA-24 has an over-specification risk (intermediate scale exact value unverifiable without prescribing easing math). CA-22 ADVISORY correctly demoted.
Story 009: ADEQUATE — CI grep boundary check. ADVISORY-until-CI-established promotion path is correctly specified.

**Why:** OQ-CA-01 (AnimatorState public inspectability) is the cross-cutting risk. If bevy_tweening 0.18 uses Observer callbacks rather than inspectable state, 8 ACs across 4 stories require reformulation.
**How to apply:** Block Story 002/005/008 BLOCKING ACs on OQ-CA-01 resolution. Do not let implementation begin on any AnimatorState-asserting test until cargo check confirms the enum name and visibility.
