# Epic: Audio System

> **Layer**: Polish
> **GDD**: `design/assets/production-handoffs/audio-production-queue-2026-05-04.md`; asset-manifest audio entries ASSET-045–051, ASSET-078–087, ASSET-123–127, ASSET-155–162, ASSET-175–181
> **Architecture Module**: `client/src/audio/` — `AudioPlugin` (new)
> **Status**: Ready
> **Stories**: 1 story (bootstrap + timer urgency cue); subsequent P0/P1 cue stories deferred

## Overview

Audio System owns client-side audio playback: plugin registration, asset loading,
and per-cue wiring from existing ECS message signals to Bevy 0.18 `AudioPlayer`
spawns. The production queue defines 47 audio assets across P0–P3 priority bands;
this epic scopes only the bootstrap slice (Story 001) needed to prove the audio
pipeline works end-to-end on one cue before committing production audio assets.

The `TimerUrgencyAudio` message already exists in `client/src/ui/hand/mod.rs` and
fires exactly once when the placement timer reaches ≤5 seconds. That signal is the
shortest path to a wired, evidenced audio cue without touching the result flow or
the Sprint 9 must-have scope.

No final audio, final mix, release readiness, full asset approval, Sound Bible
authoring, audio accessibility (volume controls, QA-COND-0005 audio rows), or
broad accessibility completion is claimed by this epic.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Audio cues are triggered by server-authoritative state delivered via S2C messages or existing client-side ECS signals; no client-originated randomness or authority | LOW |
| No audio-specific ADR yet | Bevy 0.18 audio loading strategy (Handle<AudioSource> vs typed AudioAssets collection) to be decided at Story 001 implementation | HIGH — see FLAG-2 in production queue |

## Requirements

| Source | Requirement | ADR Coverage |
|--------|-------------|--------------|
| Audio production queue FLAG-2 | Bevy 0.18 audio API (`AudioPlayer` / `PlaybackSettings`) must be verified before any audio implementation begins | None yet |
| ASSET-084 spec | Timer urgency cue must play once (one-shot, no loop, no tick sequence) when the placement timer reaches ≤5 seconds | ADR-002 |
| Asset manifest | `assets/audio/` directory must exist with OGG delivery format; WAV authoring masters are upstream production concern | None |
| Asset spec hand-ui | All hand-UI audio on `ui_hand` channel, mono, 44100 Hz | None |

## Traceability Notes

Story 001 is bootstrap infrastructure and does not yet have a dedicated
`TR-AUDIO-*` entry in `docs/architecture/tr-registry.yaml`. It is traceable to
the audio production queue priority spec and ASSET-084. A `TR-AUDIO-001` entry
should be added when the audio ADR is authored; the implementer may author the ADR
inline with Story 001 or propose it as a follow-on.

## Dependency Map

| Dependency | Existing Surface | Audio System Use |
|------------|------------------|-----------------|
| Hand UI | `TimerUrgencyAudio` message, `MessageWriter<TimerUrgencyAudio>` in `client/src/ui/hand/mod.rs` | Signal source — audio system consumes this message to trigger cue playback |
| Bevy 0.18 built-in audio | `AudioPlugin` included in `DefaultPlugins`; `AudioPlayer`, `PlaybackSettings` required components | Plugin registration and one-shot cue spawn |
| bevy_asset_loader | Currently commented out in `client/Cargo.toml`; Story 001 must decide: re-enable + pin, or load via `AssetServer` directly | Asset loading strategy for `sfx_timer_urgency_default.ogg` |
| Audio production queue | `design/assets/production-handoffs/audio-production-queue-2026-05-04.md` | ASSET-084 spec for timer urgency cue |

## Current Implementation Gaps

- `assets/audio/` directory does not exist — no OGG or WAV files present
- No `AudioPlugin` registration confirmed in `client/src/main.rs` (DefaultPlugins
  must not exclude audio; verify)
- No `AudioAssets` typed collection or `Handle<AudioSource>` load path defined
- `bevy_asset_loader` dependency is commented out in `client/Cargo.toml`
- No audio module (`client/src/audio/`) exists
- Bevy 0.18 `AudioPlayer`/`PlaybackSettings` API not yet verified against
  `liv-bevy-018` patterns (FLAG-2 HIGH risk)
- Sound Bible not yet authored for this project (template only)

## Definition of Done

- [ ] `AudioPlugin` is confirmed active in the client (DefaultPlugins includes it,
      or it is explicitly registered; no explicit exclusion).
- [ ] `assets/audio/ui/hand/sfx_timer_urgency_default.ogg` exists as a generated
      placeholder (silent or minimal OGG — not a final mixed asset).
- [ ] A load path for the timer urgency cue is defined and functional (typed
      collection or `AssetServer::load`).
- [ ] A system in `client/src/audio/` consumes `TimerUrgencyAudio` and spawns
      `(AudioPlayer::new(handle), PlaybackSettings::ONCE)`.
- [ ] The cue plays once at the ≤5s mark and does not loop.
- [ ] Graceful no-op if the asset handle is not yet loaded.
- [ ] Manual evidence document exists at `production/qa/evidence/`.
- [ ] No final audio, final mix, release readiness, full asset approval, Sound
      Bible authoring, audio accessibility (QA-COND-0005 audio rows), or broad
      accessibility completion is claimed.

## Stories

| # | Story | Type | Status | Requirement | ADR |
|---|-------|------|--------|-------------|-----|
| 001 | [Audio Bootstrap + Timer Urgency Cue](story-001-audio-bootstrap-and-timer-urgency-cue.md) | Integration | Ready | ASSET-084; audio production queue | ADR-002; audio ADR TBD |

## Next Step

Run `/story-readiness audio-system story-001` to confirm implementation readiness,
then `/dev-story audio-system/story-001` in the Codex environment where Rust code
is written. Do not run `/dev-story` in this design-only project.
