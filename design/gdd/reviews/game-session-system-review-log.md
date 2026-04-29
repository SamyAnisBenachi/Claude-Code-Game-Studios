# Review Log — Game Session System

## Review — 2026-04-29 (R2) — Verdict: MAJOR REVISION NEEDED → Revised Inline

**Scope signal:** L
**Specialists:** game-designer, systems-designer, network-programmer, qa-lead, creative-director (senior synthesis)
**Blocking items:** 16 | **Recommended:** 8+
**Prior verdict resolved:** Partially — R1 inline revisions were correct but left three R1 cleanup artifacts (GSS-23, Tuning Knobs grace-window text, Dependencies disconnect_grace_seconds) and introduced new structural gaps found in R2.

**Summary:** R2 found that R1's "disconnect = immediate cancel" change propagated to Rule 9 but not to GSS-23, the Tuning Knobs Interacts-With column, or the Dependencies table — three live contradictions. Beyond cleanup, R2 found five new structural issues: (1) Rule 9's OnDisconnected trigger is delayed 2–7 minutes in WASM/browser — heartbeat fallback needed with new `lobby_heartbeat_timeout_seconds` GameConfig field and GSS-owned tracker; (2) F4's strict `<` contradicted the "same tick, session proceeds" edge case — resolved by changing to `<=`; (3) F3 used `f32` for clock time — changed to `f64`; (4) S2CSlotUpdated referenced in Interactions table but never defined — added to Rule 2 with full-vector broadcast spec; (5) "one locked, one not" player state completely unspecified — added opponent-is-browsing animated indicator rule. New Rule 13 resolves OQ2 (one session per player, idempotent C2SCreateRoom). 16 ACs added or restructured. All 16 blockers resolved inline.

**Status after revision:** In Review — pending R3 /design-review in fresh session.

---

## Review — 2026-04-29 — Verdict: MAJOR REVISION NEEDED → Revised Inline

**Scope signal:** L
**Specialists:** game-designer, systems-designer, network-programmer, qa-lead, creative-director (senior synthesis)
**Blocking items:** 6 | **Recommended:** 8+
**Prior verdict resolved:** No — first review

**Summary:** The infrastructure layer was solid but the document contained a pillar-violating contradiction (Rule 7 broadcast-immediately vs. Player Fantasy "simultaneous reveal") and multiple implementation-detail leakages into what should be design-level rules. The most critical finding: Rule 7 was written to broadcast class identity immediately on lock, but design intent (confirmed in authoring session) was "hold reveal until all players lock, then reveal simultaneously." This would have produced wrong implementation code. All 6 blocking items were revised inline: Rule 7 rewritten to deferred simultaneous reveal (`S2CClassLocked` unicast + `S2CClassesRevealed` broadcast); Rule 9 changed from 30s grace to MVP forfeit (reconnect credential mechanism not yet designed); Rule 11 tick-level claims stripped (Observer vs. Events<T> pushed to ADR); SessionSlot.class declared canonical source; all lobby C2S/S2C messages added to network-protocol.md; stale Tuning Knobs note removed. Rule 12 (server restart behavior) added. 12 AC changes including GSS-17 split, GSS-24 downgraded, GSS-33 upgraded to BLOCKING.

**Status after revision:** In Review — pending R2 /design-review in fresh session.
