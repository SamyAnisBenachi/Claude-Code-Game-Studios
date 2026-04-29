---
name: card-data-pool QL-STORY-READY review
description: Gate review results for 6 card-data-pool epic stories — verdicts and key gaps found
type: project
---

QL-STORY-READY gate performed 2026-04-29 on card-data-pool Core epic (6 stories).

Stories 001-005: ADEQUATE (test specs produced, clarifications absorbed into specs).
Story 006: INADEQUATE — returned to programmer pending two blockers.

**Why:** Story 006 AC-1/AC-2 require verifying Lightyear unicast dispatch in a headless App, which is not possible without a Lightyear test harness. Story 006 AC-4 references `ReconnectTracker.snapshot_sent` which is not defined in any ADR or type inventory.

**How to apply:** Do not assign Story 006 to a developer until:
1. Lead Programmer decides: pure-function message assembly (Option A) vs. Lightyear test harness post-S1-05 (Option B).
2. ReconnectTracker struct is defined in an ADR or story update.

**Key clarifications absorbed into test specs (not blocking):**
- Story 002 AC-10: "all other 24 cards have total_acquired==0" must be in Given for deterministic assertion.
- Story 002 AC-3: family_index fixture must be explicitly constructed in test setup.
- Story 003 AC-3: refresh_shop return type clarified as Vec<CardId> (compact, no None padding).
- Story 004 AC-1: "mock CardCatalog" is invalid per no-mocks rule — must use fixture-built real CardCatalog.
- Story 005 AC-4: "no error sent to client" reframed as "no S2CShopSlots written to outbound message queue."

**Test file locations assigned:**
- Story 001: tests/unit/pool/pool_state_test.rs
- Story 002: tests/unit/pool/weighted_draw_test.rs
- Story 003: tests/unit/pool/refresh_shop_test.rs
- Story 004: tests/integration/pool/session_ready_test.rs
- Story 005: tests/integration/pool/manual_refresh_test.rs
- Story 006: pending story split decision
