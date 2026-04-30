# RSM Story 001 Check Evidence

Story: `production/epics/round-state-machine/story-001-state-and-events-scaffold.md`

Date: 2026-04-30

## Local Checks

### Forbidden RSM API grep

Command:

```powershell
Get-ChildItem -Path server\src\core\rsm -Recurse | Select-String -Pattern 'EventWriter|EventReader|Events<|add_event|derive\(States\)'
```

Result: PASS - zero matches.

### Bevy 0.18 API source check

Local crate source confirms:

- `bevy_app-0.18.1/src/app.rs` exposes `App::add_message`.
- `bevy_app-0.18.1/src/app.rs` exposes `App::add_observer`.
- `bevy_ecs-0.18.1/src/message/messages.rs` exposes `Messages<T>`.

### Cargo check

Command:

```powershell
C:\Users\Sam\.cargo\bin\cargo.exe check --workspace
```

Result: BLOCKED locally - MSVC `link.exe` is not available in this shell, so Cargo cannot compile build scripts. CI must provide the compile proof for this story.
