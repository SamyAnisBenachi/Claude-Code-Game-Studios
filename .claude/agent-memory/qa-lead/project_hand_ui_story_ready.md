---
name: hand-ui QL-STORY-READY review
description: Story readiness verdicts for all 13 Hand UI epic stories (HU-001 through HU-013)
type: project
---

Stories 001/002/005/006/007/010/011/013 ADEQUATE.
Stories 003/004/008/009 have GAPS.
Story 012 has GAPS (HU-28/28b DEFERRED pending OQ8; HU-29 ADEQUATE).

**Why:** Reviewed 2026-05-01 for sprint readiness per QL-STORY-READY protocol.

**Gap details:**
- HU-003: Animator<T> type list deferred to implementation; C2S message queue accessibility unconfirmed in test context
- HU-004: HU-30 HandFullNotification is a DESIGN AMBIGUITY (runtime spawn vs pre-pooled violates ADR-021 pre-pooling rule); player.gold resource type unnamed
- HU-008: HU-21b fan zone boundary undefined — must be a testable resource (Res<FanZoneBounds>)
- HU-009: Grace window timing sequence (HU-15/15b) needs explicit virtual time step ordering in integration harness
- HU-012: HU-28/28b blocked by OQ8 (S2CActivationRejected not in NP GDD); HU-29 alone is adequate

**How to apply:** Before sprint opens: designer resolves HU-30 ambiguity (pre-pool or spawn), HU-21b zone definition, OQ8 NP registration, player.gold/reserve_mana resource types, Submit string format ("1 card" vs "1 cards"), and timer=0 reconnect edge case.
