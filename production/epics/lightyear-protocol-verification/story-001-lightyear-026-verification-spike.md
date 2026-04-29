# Story 001: Lightyear 0.26 Verification Spike ⭐

> **Epic**: Lightyear Protocol & Verification Spike
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Integration
> **Manifest Version**: 2026-04-29
> **Priority**: SPRINT 1 STORY 1.0 — Must be the first story implemented this sprint

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: TR-??? (covers all 20 control-manifest.md Lightyear 0.26 checklist items; ADR-012 open condition)

**ADR Governing Implementation**: ADR-008: Lightyear Channel Config + ADR-012: Session-Ready Delivery
**ADR Decision Summary**: All Lightyear 0.26 API surface (channel registration, MessageSender/MessageReceiver, NetworkTarget unicast shape, Observer ordering) must be verified against docs.rs before any networking code is written. ADR-012 has an open condition requiring a unit test of Commands::trigger() flush ordering.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: **HIGH**
**Engine Notes**: Lightyear 0.26 is entirely post-training-cutoff (released January 2026). Every API assumption in ADR-008 through ADR-012 carries "VERIFY BEFORE IMPLEMENTING" flags. This story IS that verification. No implementation assumptions from prior training data should be trusted without checking docs.rs for the 0.26 release.

**Control Manifest Rules (Foundation layer)**:
- Required: Exactly two Lightyear channels: `ReliableChannel` and `UnreliableChannel`. Channel assignment is permanent per message type.
- Required: All channel definitions live in `shared/src/protocol.rs`. Both server and client compile against identical channel types.
- **Manifest gate**: "Do not merge any networking story with unverified checklist items."

---

## Acceptance Criteria

**Verification report (all 20 items):**
- [ ] `tests/evidence/lightyear-026-verification.md` exists and contains all 20 items from `docs/architecture/control-manifest.md §Lightyear 0.26 Verification Checklist`
- [ ] Each item is annotated with either `✅ CONFIRMED` (API exists as assumed) or `⚠️ DIFFERS — [note]` (API differs from ADR assumption, with the correct API and a resolution path documented)
- [ ] Items 1–3 (channel definition syntax, ChannelMode, ChannelDirection): verified
- [ ] Items 4–6 (MessageSender/MessageReceiver types, send/receive method names): verified
- [ ] Items 7–9 (NetworkTarget variants, server unicast send API): verified
- [ ] Item 10 (in-order delivery guarantee on reliable channel): verified
- [ ] Items 11–14 (reconnect/snapshot ordering, ClientId reassignment, OnConnected timing): verified
- [ ] Items 15–17 (Commands::trigger Observer flush, resource visibility in Observer handler, Trigger<T> type): verified
- [ ] Items 18–20 (component replication opt-in, ReplicationGroup API, LocalTimeline Resource): verified

**ADR-012 open condition:**
- [ ] Unit test written at `server/tests/session_ready_observer_test.rs` (or `tests/unit/foundation/`) that verifies `Commands::trigger()` flush ordering: a Bevy App with a registered Observer for `SessionReady` receives the event in the same frame as the trigger
- [ ] The test asserts that `Res<SessionConfig>` inserted via `Commands::insert_resource()` BEFORE `Commands::trigger(SessionReady)` is visible to the Observer handler
- [ ] **If the test passes**: document "ADR-012 open condition: RESOLVED — flush ordering confirmed, no `apply_deferred` needed" in the verification report
- [ ] **If the test fails**: add `apply_deferred` to the `.chain()` in `RsmPlugin::build()` sketch (document the fix path in the verification report; actual implementation deferred to GSS epic which owns `RsmPlugin`)

**Checklist updates:**
- [ ] All 20 items in `docs/architecture/control-manifest.md` updated from `⬜` to `✅` (CONFIRMED) or `⚠️` (DIFFERS) based on verification findings
- [ ] Any DIFFERS items include a concrete resolution path — not just "investigate further"

---

## Implementation Notes

*Derived from ADR-008 §Engine Compatibility and all ADR "Verification Required" sections:*

**How to verify each item:**
1. Open `https://docs.rs/lightyear/0.26.0/lightyear/` (or the exact 0.26.x patch)
2. For each item, search the docs for the named type/method/enum
3. Confirm: does it exist? Is the signature as assumed? Is the module path correct?
4. Record: "CONFIRMED: `lightyear::prelude::MessageSender<T>` exists, signature matches ADR-008" or "DIFFERS: unicast API is `ServerMessages::send_message` not `send_message_to_target` — see [link]"

**Key items likely to differ (ADR flags as HIGH risk):**
- **Item 1**: Channel registration — Lightyear 0.26 may use `app.add_channel::<T>(ChannelSettings { ... })` rather than a macro. Verify exact syntax.
- **Item 4**: `MessageSender<T>` / `MessageReceiver<T>` — these type names may differ in 0.26. Check `lightyear::prelude::*` exports.
- **Item 7**: `NetworkTarget::Single(ClientId)` — variant may be `NetworkTarget::Single(ClientId)` or `NetworkTarget::Only([id])` or different. Critical for ADR-001 unicast.
- **Item 9**: Server send API — `ConnectionManager::send_message_to_target` or similar — verify exact path.
- **Item 15/16**: `Commands::trigger()` + Observer flush ordering in Bevy 0.18 — this determines whether ADR-012's `apply_deferred` guard is needed.

**ADR-012 open condition test sketch:**
```rust
#[derive(Event)]
struct SessionReady;

#[derive(Resource)]
struct SessionConfig { value: u32 }

#[test]
fn test_session_ready_observer_flush_ordering() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let config_visible = Arc::new(AtomicBool::new(false));
    let flag = config_visible.clone();

    app.observe(move |_trigger: Trigger<SessionReady>, config: Res<SessionConfig>| {
        flag.store(config.value == 42, Ordering::SeqCst);
    });

    app.add_systems(Update, |mut commands: Commands| {
        commands.insert_resource(SessionConfig { value: 42 });
        commands.trigger(SessionReady);
    });

    app.update();
    assert!(config_visible.load(Ordering::SeqCst),
        "ADR-012 FAIL: SessionConfig not visible in SessionReady Observer — add apply_deferred");
}
```
If this test fails, the fix is documented (but not implemented here — GSS epic owns `RsmPlugin::build()`).

**Where to find Lightyear 0.26 docs:**
- Primary: `https://docs.rs/lightyear/0.26.0`
- Release notes: `https://github.com/cBournhonesque/lightyear/releases`
- Migration guide (if exists): check the lightyear repo CHANGELOG.md for 0.26
- Examples: `https://github.com/cBournhonesque/lightyear/tree/main/examples`

---

## Out of Scope

- Story 002: Defining all message types — this story only verifies the API surface
- Story 003: Plugin implementation — verification first, implementation second
- Story 004: End-to-end connection test

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: All 20 items annotated**
  - Given: `tests/evidence/lightyear-026-verification.md` written
  - When: File is read
  - Then: Contains exactly 20 numbered items; each has CONFIRMED or DIFFERS annotation; no ⬜ items remain

- **AC: ADR-012 test written and run**
  - Given: `server/tests/session_ready_observer_test.rs` written
  - When: `cargo test -p server session_ready_observer` is run
  - Then: Either PASS (Observer sees resource → no apply_deferred needed) or FAIL (documented fix path written)

- **AC: control-manifest.md updated**
  - Given: Verification report complete
  - When: `docs/architecture/control-manifest.md §Lightyear 0.26 Verification Checklist` is read
  - Then: All 20 items show ✅ or ⚠️ — zero ⬜ remain

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/evidence/lightyear-026-verification.md` — all 20 items annotated
- `server/tests/session_ready_observer_test.rs` — test written and result documented
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `workspace-and-shared-types` Story 004 Done (Lightyear deps present in Cargo.toml so docs.rs lookup matches actual dependency)
- Unlocks: **Story 002, Story 003, Story 004** — AND all Core/Feature epics that touch networking. **This story is a hard gate.**

## Completion Notes
**Completed**: 2026-04-29
**Criteria**: 9/9 passing
**Deviations**: ADVISORY — Item 17 corrected: `Trigger<T>`→`On<T>`, `App::observe()`→`App::add_observer()` (Bevy 0.16+ rename; proved by CI compilation)
**Test Evidence**: Integration — `tests/evidence/lightyear-026-verification.md` (20 items); `server/tests/session_ready_observer_test.rs` (2 tests, CI PASS run 25133926012)
**ADR-012**: RESOLVED — flush ordering confirmed, no apply_deferred needed
**Code Review**: Skipped (Lean mode)
