# Story 001: Audio Bootstrap + Timer Urgency Cue

> **Epic**: Audio System
> **Status**: Ready
> **Layer**: Polish
> **Type**: Integration
> **Manifest Version**: 2026-05-08

## Context

**GDD**: None. Traced to `design/assets/production-handoffs/audio-production-queue-2026-05-04.md`
(ASSET-084: Timer Urgency SFX) and `design/assets/asset-manifest.md`.
**Requirement**: ASSET-084 — timer urgency cue plays once (one-shot) when placement
timer reaches ≤5 seconds. No loop, no tick sequence.
**ADR Governing Implementation**: [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md).
Audio-specific ADR not yet authored — implementer must verify Bevy 0.18 audio
loading strategy (see FLAG-2 below) and may propose an ADR inline.

The placement timer urgency signal already exists. `client/src/ui/hand/mod.rs`
defines `TimerUrgencyAudio` (a `Message`, zero-copy, derives `Default`) and
writes it exactly once via `MessageWriter<TimerUrgencyAudio>` when the timer
crosses the 5-second threshold. No audio playback code currently exists.

This story bootstraps the minimal audio pipeline end-to-end:

1. Confirm `AudioPlugin` is active in the client build.
2. Place a generated placeholder OGG at the ASSET-084 path.
3. Define a load path for that single asset.
4. Author a system that consumes `TimerUrgencyAudio` and spawns a one-shot
   `AudioPlayer` entity.
5. Produce manual evidence that the cue fires once at ≤5s and does not loop.

This story is an optional Sprint 9 audio slice. It does not touch the result flow,
reconnect logic, or any must-have Sprint 9 stories. Pull it only if there is sprint
capacity after S9-RS-002 and S9-NATIVE-001 are underway.

**⚠️ FLAG-2 (HIGH RISK):** Bevy 0.18 audio API — `AudioPlayer` and
`PlaybackSettings` are Required Components in 0.18; `AudioBundle` and the pre-0.15
`AudioSource` component patterns no longer exist. Activate `liv-bevy-018` before
writing any audio code. Verify `DefaultPlugins` does not exclude `AudioPlugin` in
`client/src/main.rs`.

## Traceability Gap

No `TR-AUDIO-*` entry exists in `docs/architecture/tr-registry.yaml`. This story
does not create one. It is traceable to ASSET-084 in the audio production queue.
The implementer should author `TR-AUDIO-001` when the audio ADR is written.

## Acceptance Criteria

- [ ] `AudioPlugin` is confirmed active: `DefaultPlugins` in `client/src/main.rs`
      does not exclude `AudioPlugin`, OR `AudioPlugin` is explicitly added.
- [ ] `assets/audio/ui/hand/sfx_timer_urgency_default.ogg` exists as a generated
      placeholder file (silent or near-silent ≤0.5s OGG; not a final mixed asset;
      not a production sound design deliverable).
- [ ] A load path for the placeholder is defined in the client: either a typed
      `AudioAssets` collection via `bevy_asset_loader` (if enabled and pinned for
      Bevy 0.18) or a `Handle<AudioSource>` loaded via `AssetServer::load` in a
      startup or `OnEnter(ClientState::InSession)` system.
- [ ] `client/src/audio/mod.rs` (or equivalent) defines `AudioSystemPlugin` and
      is registered in the client app.
- [ ] A system in `AudioSystemPlugin` reads `MessageReader<TimerUrgencyAudio>`
      (Bevy 0.18 / Lightyear 0.26 message pattern) and, when the message is
      received, spawns `(AudioPlayer::new(handle.clone()), PlaybackSettings::ONCE)`.
- [ ] The cue spawns exactly once per `TimerUrgencyAudio` message — no loop, no
      repeated firing, no tick sequence.
- [ ] If the asset handle is not yet loaded (e.g., during an early-session edge
      case), the system is a graceful no-op rather than a panic or `unwrap`.
- [ ] `client/src/lib.rs` or the top-level client plugin registration includes
      `AudioSystemPlugin`.
- [ ] Manual evidence document exists at
      `production/qa/evidence/audio-timer-urgency-YYYY-MM-DD.md` and records:
      cue fires at ≤5s mark, no crash, no loop, placeholder is audible or
      confirmed silent as expected.

## Implementation Notes

- **Activate `liv-bevy-018` before writing any `.rs` file.** Bevy 0.18 audio
  patterns differ significantly from pre-0.15. Do not use `AudioBundle`,
  `AudioSource` as a component, `play()`, or `EventWriter<PlayAudio>`.
- **Bevy 0.18 pattern**: spawn `(AudioPlayer::new(handle), PlaybackSettings::ONCE)`
  as an entity. `PlaybackSettings::ONCE` plays once and the entity can be
  despawned after or left to auto-despawn if Bevy handles completed one-shots.
  Verify auto-despawn behavior with `liv-bevy-018`.
- **Message pattern**: Use `MessageReader<TimerUrgencyAudio>` (Lightyear 0.26)
  consistent with the existing Hand UI message registration at
  `client/src/ui/hand/mod.rs:783` (`.add_message::<TimerUrgencyAudio>()`).
  Activate `liv-bevy-lightyear` alongside `liv-bevy-018`.
- **Asset loading strategy choice** (decide at implementation time):
  - Option A — `AssetServer::load` in a startup system: simpler, no extra
    dependency change. Load `"audio/ui/hand/sfx_timer_urgency_default.ogg"` and
    store the handle in a `Local` or `Resource`.
  - Option B — Re-enable `bevy_asset_loader`: pin a Bevy 0.18-compatible version
    in `client/Cargo.toml` and define a typed `AudioAssets` collection. More
    future-proof for the full 47-asset catalog, but adds dependency friction.
  - Do not choose Option B without first verifying the compatible
    `bevy_asset_loader` version on crates.io for Bevy 0.18.
- **Placeholder OGG**: generate a silent or near-silent OGG (e.g., 0.3s silence,
  44100 Hz, mono) using `ffmpeg` or equivalent. This is not a production asset.
  Mark it clearly in git commit message as a placeholder pending audio-director
  delivery.
- **No hardcoded paths in systems**: expose the asset path as a constant in
  `client/src/audio/mod.rs` so it matches the production queue filename exactly.
- **No `unwrap()`** on the asset handle in production paths. Use `.expect("audio
  assets must be loaded")` only if loading is guaranteed by a state gate, or use
  a conditional check.
- **Channel / bus routing**: the production spec assigns hand-UI audio to the
  `ui_hand` bus. Bevy 0.18 audio bus routing is a separate concern; do not block
  this story on bus implementation. Add a `// TODO: route to ui_hand bus` comment
  if bus support is deferred.

## Out of Scope

- Final mixed or production-approved audio assets (ASSET-084 placeholder only)
- Sound Bible authoring
- Any P0 board cues (ASSET-045–050) or P0 combat cues (ASSET-156–160)
- Remaining P1 hand-UI cues (ASSET-078–083, ASSET-085–087)
- P2/P3 auction, class, lobby, or prism cues
- Audio bus routing or volume mixing infrastructure
- Volume controls, mute toggle, or audio accessibility rows (QA-COND-0005)
- Background music or adaptive audio
- AudioDirector / Sound Bible direction for final sound character
- Automated unit or integration tests for audio playback (visual/feel type —
  manual evidence is sufficient)
- Full asset approval or release readiness
- Broad accessibility completion
- Sprint 9 must-have stories (S9-RS-002, S9-NATIVE-001, S9-RS-003, S9-QA-001)

## QA Test Cases

Manual — Story type is Visual/Feel (advisory gate).

- **Cue fires at threshold**
  - Given: client is in session, placement timer is running
  - When: the timer crosses below 5 seconds
  - Then: the timer urgency cue plays once (audible or confirmed silent placeholder)
    and does not repeat or loop

- **Cue does not fire twice**
  - Given: `TimerUrgencyAudio` fires once per urgency threshold crossing
  - When: the same timer urgency event occurs in one placement phase
  - Then: audio plays exactly once; no second spawn from the same event

- **Graceful no-op when asset absent**
  - Given: the OGG placeholder file is temporarily removed from the asset path
  - When: `TimerUrgencyAudio` message arrives
  - Then: no panic or `unwrap` crash; the system skips silently

## Test Evidence

**Type**: Visual/Feel → **ADVISORY** gate (not blocking).

Required artifact: `production/qa/evidence/audio-timer-urgency-YYYY-MM-DD.md`

Evidence document must record:
- Session date and build (native or browser/WASM)
- Whether the cue audibly fired at the ≤5s timer mark
- Whether the cue looped or fired more than once (expected: no)
- Whether any crash or panic occurred (expected: none)
- Whether the placeholder was confirmed silent or audible
- Non-claims: not a final audio review, not a mix approval, not release readiness

Status: **Not yet created** — created by implementer during manual walkthrough.

## Dependencies

**Depends on**:
- `TimerUrgencyAudio` message registered in `client/src/ui/hand/mod.rs` (confirmed
  present at line 783)
- `client/src/main.rs` — must not exclude `AudioPlugin` from `DefaultPlugins`
- `assets/audio/ui/hand/sfx_timer_urgency_default.ogg` placeholder (created by
  this story)

**Unlocks**:
- Subsequent audio cue stories for P0 board SFX (ASSET-045–050)
- Subsequent audio cue stories for P1 hand-UI SFX (ASSET-078–083, ASSET-085–087)
- Audio bus routing story (future, not yet scoped)
- Sound Bible authoring (can proceed independently; does not block this story)
