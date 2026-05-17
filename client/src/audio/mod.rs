use bevy::prelude::*;

use crate::ui::hand::TimerUrgencyAudio;

const ASSET_PATH: &str = "audio/ui/hand/sfx_timer_urgency_default.ogg";

#[derive(Resource)]
pub struct TimerUrgencyAudioHandle(pub Handle<AudioSource>);

pub struct AudioSystemPlugin;

impl Plugin for AudioSystemPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("AudioSystemPlugin loaded");
        app.add_systems(Startup, load_timer_urgency_audio)
            .add_systems(Update, play_timer_urgency_cue);
    }
}

fn load_timer_urgency_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<AudioSource> = asset_server.load(ASSET_PATH);
    commands.insert_resource(TimerUrgencyAudioHandle(handle));
}

pub fn play_timer_urgency_cue(
    _handle_res: Option<Res<TimerUrgencyAudioHandle>>,
    mut reader: MessageReader<TimerUrgencyAudio>,
) {
    // PROMPT 1017 crash guard (re-land of PROMPT 996): drain
    // TimerUrgencyAudio without spawning an AudioPlayer. The placeholder
    // asset at `audio/ui/hand/sfx_timer_urgency_default.ogg` is Ogg Opus
    // (verified via `file` -- 134 B, "Ogg data, Opus audio, version 0.1,
    // mono, 44100 Hz"). rodio/Bevy 0.18 has no Opus decoder available even
    // with `bevy/vorbis` enabled, so spawning an AudioPlayer for this
    // handle panics Bevy's internal `play_queued_audio_system<AudioSource>`
    // with `Err(UnrecognizedFormat)` once the 5s placement-timer urgency
    // threshold fires. Playback must only be re-enabled after the
    // placeholder is re-encoded as a supported codec (Vorbis, MP3, FLAC,
    // or WAV) AND the matching `bevy/<codec>` feature is added to
    // `client/Cargo.toml`. See `tests/integration/playable_client/
    // timer_urgency_audio_crash_guard_test.rs` for the pinned behavior.
    for _ in reader.read() {}
}
