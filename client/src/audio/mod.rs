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
    // PROMPT 996 crash guard: drain TimerUrgencyAudio without spawning an
    // AudioPlayer. The placeholder asset at `audio/ui/hand/sfx_timer_urgency_default.ogg`
    // is Ogg Opus, which rodio/Bevy 0.18 cannot decode -- spawning an
    // AudioPlayer for it panics `play_queued_audio_system` with
    // `Err(UnrecognizedFormat)` once the 5s placement-timer urgency threshold
    // fires. Playback must only be re-enabled after the placeholder is
    // replaced with a supported codec (e.g. Vorbis, MP3 with `mp3` feature).
    for _ in reader.read() {}
}
