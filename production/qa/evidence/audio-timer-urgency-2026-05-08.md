# Audio Timer Urgency — Test Evidence

**Story**: S9-AUDIO-001  
**Date**: 2026-05-08  
**Gate**: ADVISORY (Visual/Feel story type — not blocking)

---

## AudioPlugin Status

`DefaultPlugins` is used in `client/src/main.rs` without excluding `AudioPlugin`.
`bevy_audio` feature added to `client/Cargo.toml` (the `"2d"` feature collection
does not include audio). `bevy_audio v0.18.1` confirmed present in `cargo check`
output.

## Placeholder OGG

- Path: `assets/audio/ui/hand/sfx_timer_urgency_default.ogg`
- Size: 134 bytes
- Format: OGG container with OpusHead + OpusTags headers + minimal audio page + EOS
- Generated: PowerShell OGG binary construction (ffmpeg not available on build machine)
- Magic bytes verified: `4F 67 67 53` ("OggS") ✓
- Note: minimal Opus frame — placeholder only; audibility at runtime depends on
  codec-level decoding. Load failure is handled gracefully (no panic).

## AudioSystemPlugin

Registered in `client/src/main.rs`. Defined in `client/src/audio/mod.rs`.

- Startup system loads `"audio/ui/hand/sfx_timer_urgency_default.ogg"` via
  `AssetServer::load` and inserts `TimerUrgencyAudioHandle` resource.
- Update system reads `MessageReader<TimerUrgencyAudio>` and spawns
  `(AudioPlayer::new(handle.clone()), PlaybackSettings::ONCE)` per message.
- Graceful no-op: uses `Option<Res<TimerUrgencyAudioHandle>>` — returns early
  if resource absent. No `unwrap()` in production path.
- `// TODO: route to ui_hand bus` comment defers bus routing per story scope.

## Build Verification

```
cargo fmt -p client -- --check     PASS
cargo check -p client              PASS (bevy_audio v0.18.1 compiled)
cargo check --workspace            PASS
git diff --check origin/main...HEAD  PASS
```

## Manual Playthrough

Pending — requires a playable native or WASM build session. Full walkthrough
(fire at ≤5s mark, confirm no loop, confirm no crash) to be completed when
a play session is available.

## Non-Claims

- This is **NOT** a final audio review.
- This is **NOT** mix approval.
- This is **NOT** a Sound Bible delivery.
- This is **NOT** release readiness.
- Placeholder OGG only — production asset delivery pending audio-director handoff.
- No broad accessibility closure.
