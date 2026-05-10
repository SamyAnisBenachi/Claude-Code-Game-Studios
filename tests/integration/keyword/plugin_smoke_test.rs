use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use server::feature::keyword::{ChainDeathBuffer, KeywordPlugin, KeywordTriggered};
use shared::keyword::KeywordPayload;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn write_keyword_triggered(mut writer: MessageWriter<KeywordTriggered>) {
    writer.write(KeywordTriggered {
        source_unit_id: None,
        sub_step: 1,
        payload: KeywordPayload::ShieldConsumed,
    });
}

#[test]
fn keyword_plugin_registers_keyword_triggered_message() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(KeywordPlugin);
    app.finish();
    app.cleanup();

    assert!(app
        .world()
        .get_resource::<Messages<KeywordTriggered>>()
        .is_some());

    app.world_mut()
        .run_system_once(write_keyword_triggered)
        .expect("KeywordTriggered MessageWriter should run");

    let messages = app.world().resource::<Messages<KeywordTriggered>>();
    let mut cursor = messages.get_cursor();
    let written: Vec<_> = cursor.read(messages).collect();

    assert_eq!(written.len(), 1);
    assert_eq!(written[0].sub_step, 1);
    assert!(matches!(written[0].payload, KeywordPayload::ShieldConsumed));
}

#[test]
fn keyword_plugin_initialises_empty_chain_death_buffer() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(KeywordPlugin);
    app.finish();
    app.cleanup();

    let buffer = app.world().resource::<ChainDeathBuffer>();
    assert!(buffer.0.is_empty());
}
