//! PROMPT 996 regression: `play_timer_urgency_cue` must consume
//! `TimerUrgencyAudio` messages without spawning an `AudioPlayer`.
//!
//! Background: PROMPT 994 traced the user-reported "Select a card" crash
//! to client-side Bevy audio, not draft UI / server / Lightyear. The
//! placement timer urgency threshold writes `TimerUrgencyAudio` at 5s
//! remaining, and the audio module previously spawned `AudioPlayer` for
//! `audio/ui/hand/sfx_timer_urgency_default.ogg`. That placeholder is
//! Ogg Opus, which rodio/Bevy 0.18 cannot decode -- Bevy's internal
//! `play_queued_audio_system<AudioSource>` then panics with
//! `Err(UnrecognizedFormat)`. Both clients in the live two-client run
//! reproduced this deterministically after Placement entry > 6s.
//!
//! Crash guard: this test pins the disarmed behavior. Until a future
//! story replaces the placeholder with a supported codec and re-enables
//! playback, the handler must drain messages without spawning playback
//! entities or panicking.

use bevy::prelude::*;
use client::audio::play_timer_urgency_cue;
use client::ui::hand::TimerUrgencyAudio;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn prompt_996_timer_urgency_cue_drains_messages_without_audio_player_spawn() {
    test_helpers::init_test_tracing();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<TimerUrgencyAudio>();
    app.add_systems(Update, play_timer_urgency_cue);

    let entities_before = app
        .world_mut()
        .query::<Entity>()
        .iter(app.world())
        .count();

    app.world_mut().write_message(TimerUrgencyAudio);
    app.world_mut().write_message(TimerUrgencyAudio);
    app.world_mut().write_message(TimerUrgencyAudio);

    app.update();

    let audio_players: usize = app
        .world_mut()
        .query::<&AudioPlayer<AudioSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        audio_players, 0,
        "play_timer_urgency_cue must not spawn AudioPlayer entities while \
         the placeholder Ogg Opus asset is unsupported by rodio/Bevy 0.18"
    );

    let entities_after = app
        .world_mut()
        .query::<Entity>()
        .iter(app.world())
        .count();
    assert_eq!(
        entities_after, entities_before,
        "no entities should be spawned in response to TimerUrgencyAudio \
         messages while the audio playback path is disarmed (PROMPT 996)"
    );

    for _ in 0..3 {
        app.world_mut().write_message(TimerUrgencyAudio);
        app.update();
    }

    let audio_players_after_runs: usize = app
        .world_mut()
        .query::<&AudioPlayer<AudioSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        audio_players_after_runs, 0,
        "additional urgency messages across subsequent frames must remain \
         disarmed -- no AudioPlayer spawn under any tick cadence"
    );
}
