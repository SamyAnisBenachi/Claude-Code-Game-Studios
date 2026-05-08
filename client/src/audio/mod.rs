use bevy::prelude::*;

use crate::ui::hand::TimerUrgencyAudio;

const ASSET_PATH: &str = "audio/ui/hand/sfx_timer_urgency_default.ogg";

#[derive(Resource)]
pub struct TimerUrgencyAudioHandle(pub Handle<AudioSource>);

pub struct AudioSystemPlugin;

impl Plugin for AudioSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_timer_urgency_audio)
            .add_systems(Update, play_timer_urgency_cue);
    }
}

fn load_timer_urgency_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<AudioSource> = asset_server.load(ASSET_PATH);
    commands.insert_resource(TimerUrgencyAudioHandle(handle));
}

fn play_timer_urgency_cue(
    mut commands: Commands,
    handle_res: Option<Res<TimerUrgencyAudioHandle>>,
    mut reader: MessageReader<TimerUrgencyAudio>,
) {
    let Some(handle_res) = handle_res else {
        return;
    };
    for _ in reader.read() {
        // TODO: route to ui_hand bus
        commands.spawn((
            AudioPlayer::new(handle_res.0.clone()),
            PlaybackSettings::ONCE,
        ));
    }
}
