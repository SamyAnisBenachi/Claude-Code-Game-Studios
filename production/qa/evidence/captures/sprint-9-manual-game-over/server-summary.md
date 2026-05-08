# S9-QA-001 Server Summary

## Launch Result

| Field | Value |
|---|---|
| Binary | target/msvc-local/debug/server.exe |
| Command | `SERVER_PORT=5000 target/msvc-local/debug/server.exe` |
| Commit | d2ac17ccb2c19be18dd1b2de63d2f2e968c235c8 |
| Run duration | 8 seconds (killed by timeout tool; not a crash) |
| Exit code | 124 (timeout signal — SIGTERM) |
| Panic observed | None |
| Immediate crash | None |

## Log Capture Note

Bevy's logging on Windows writes to the Windows console API rather than to
the redirected file descriptor used by this shell environment. The file
`server-startup.log` is empty as a result. This is a capture tooling
limitation, not a server failure.

**Evidence claim**: The server binary compiled cleanly (cargo check PASS) and
ran for 8 seconds without panicking or exiting with an error code. This is
consistent with a clean startup.

**Not claimed**: Visual confirmation of server listening on port 5000, room
create/join messages, phase transitions, `S2CGameOver` observation, or
acknowledgement handling. Those require the manual two-client route, which
was not executed in this run (see defects.md MANUAL-FG-001).

## Server Startup Steps Not Observed

| Expected step | Status |
|---|---|
| Server listens on SERVER_PORT | Not confirmed — log capture failed |
| Room created on Client A join | Not reached — clients not launched |
| `S2CRoomCreated` / `S2CJoinAck` | Not reached |
| Phase changes through session | Not reached |
| `S2CGameOver` broadcast | Not reached |
| Acknowledgement handling | Not reached |
| Graceful shutdown | Not reached |
