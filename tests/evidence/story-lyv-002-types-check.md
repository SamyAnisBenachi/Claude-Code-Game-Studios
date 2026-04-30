# S2-06 Protocol Types Check

> Story: `production/epics/lightyear-protocol-verification/story-002-all-protocol-message-types.md`
> Owner: `codex-s2-06-protocol`
> Date: 2026-04-30

## Scope Verified

- `shared/src/protocol.rs` defines the S2-06 C2S/S2C message catalogue.
- `ReliableChannel` and `UnreliableChannel` are the only channel marker types.
- `register_protocol` records channel assignment and direction for all S2-06 messages.
- `C2SHeartbeat` and `S2CHeartbeat` are assigned to `UnreliableChannel`; all other messages are assigned to `ReliableChannel`.

## Lightyear 0.26 API Constraint

`tests/evidence/lightyear-026-verification.md` confirms the real Lightyear 0.26 registration API:

- items 1-2: channels are plain structs registered with `app.add_channel::<T>(ChannelSettings { mode, ..default() })`
- item 3: message direction is configured with `app.register_message::<M>().add_direction(NetworkDirection::...)`

This story was implemented as a dependency-free registration manifest in `shared/` because the active CI gate forbids `bevy_ecs`, Bevy plugin code, and Lightyear transport dependencies in the shared crate. The server/client Lightyear wiring story should adapt the manifest to the verified API calls.

## Local Command Results

Command:

```powershell
C:\Users\Sam\.cargo\bin\cargo.exe check -p shared
```

Result:

```text
Finished `dev` profile [optimized + debuginfo] target(s) in 1.05s
```

Command:

```powershell
C:\Users\Sam\.cargo\bin\cargo.exe test -p shared
```

Result:

```text
test result: ok. 0 passed; 0 failed; 0 ignored
Doc-tests shared
Finished `test` profile [optimized + debuginfo] target(s) in 1.69s
```

## Verdict

PASS locally for `shared`.
