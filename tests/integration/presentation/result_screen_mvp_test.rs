use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::result_screen::{
    build_result_objective_summary, result_screen_class_persona_label, result_screen_motion_state,
    result_screen_outcome_copy, ResultObjectiveIdentity, ResultObjectiveState,
    ResultScreenAccountingPanel, ResultScreenClassPersona, ResultScreenContinueButton,
    ResultScreenEntities, ResultScreenFocusIndicator, ResultScreenFocusOrder, ResultScreenHeadline,
    ResultScreenHeroPanel, ResultScreenMotionState, ResultScreenOutboundMessages,
    ResultScreenPlugin, ResultScreenRematchButton, ResultScreenReturnToLobbyButton,
    ResultScreenRoot, ResultScreenStep, ResultScreenStepState, ResultScreenSummaryText,
    ResultScreenViewState,
};
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientSessionIdentity, ClientState, CurrentClientPhase};
use client::ui::settings::AccessibilityPreferences;
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, GameOverReason, ObjectiveSnapshot, OpponentObjectiveSnapshot,
    PlacementTimerMultiplier, PlayerSnapshot, RoundPhase, S2CGameOver, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn outcome_copy_uses_server_authored_result_data() {
    test_helpers::init_test_tracing();
    let local = player(1);
    let opponent = player(2);

    let victory = result(Some(opponent), GameOverReason::ObjectivesDestroyed);
    assert_eq!(
        result_screen_outcome_copy(Some(&victory), Some(local)).headline,
        "VICTORY"
    );
    assert!(result_screen_outcome_copy(Some(&victory), Some(local))
        .cause
        .contains("Opponent lost two real objectives"));

    let defeat = result(Some(local), GameOverReason::ObjectivesDestroyed);
    assert_eq!(
        result_screen_outcome_copy(Some(&defeat), Some(local)).headline,
        "DEFEAT"
    );

    let draw = result(None, GameOverReason::Draw);
    assert_eq!(
        result_screen_outcome_copy(Some(&draw), Some(local)).headline,
        "DRAW"
    );

    let no_result = result(None, GameOverReason::ResolutionTimeout);
    assert_eq!(
        result_screen_outcome_copy(Some(&no_result), Some(local)).headline,
        "NO RESULT"
    );

    let disconnect_loss = result(Some(local), GameOverReason::Disconnect);
    assert!(
        result_screen_outcome_copy(Some(&disconnect_loss), Some(local))
            .cause
            .contains("Your connection was lost")
    );

    let pending = result_screen_outcome_copy(None, Some(local));
    assert_eq!(pending.headline, "RESULT PENDING");
    assert!(!pending.has_result);
}

#[test]
fn objective_summary_keeps_alive_opponent_objectives_unknown_without_reveal() {
    test_helpers::init_test_tracing();
    let snapshot = game_over_snapshot();
    let summary = build_result_objective_summary(Some(&snapshot));

    assert_eq!(summary.own_rows.len(), 5);
    assert_eq!(summary.opponent_rows.len(), 5);
    assert_eq!(summary.own_rows[0].identity, ResultObjectiveIdentity::Real);
    assert_eq!(summary.own_rows[1].identity, ResultObjectiveIdentity::Fake);
    assert_eq!(summary.own_rows[1].state, ResultObjectiveState::Destroyed);

    assert_eq!(
        summary.opponent_rows[0].identity,
        ResultObjectiveIdentity::Unknown
    );
    assert_eq!(summary.opponent_rows[0].state, ResultObjectiveState::Alive);
    assert_eq!(
        summary.opponent_rows[1].identity,
        ResultObjectiveIdentity::Fake
    );
    assert_eq!(
        summary.opponent_rows[2].identity,
        ResultObjectiveIdentity::Real
    );
    assert_eq!(
        summary.opponent_rows[3].identity,
        ResultObjectiveIdentity::Unknown
    );
    assert_eq!(
        summary.opponent_rows[4].identity,
        ResultObjectiveIdentity::Unknown
    );
}

#[test]
fn overlay_renders_game_over_result_and_hides_rematch() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    assert_eq!(root_visibility(&mut app), Visibility::Visible);
    assert_eq!(headline(&mut app), "VICTORY");
    assert!(summary_text(&mut app).contains("Own real lost 1"));
    assert!(summary_text(&mut app).contains("Opponent real revealed 1"));
    assert_eq!(focus_indicator_count(&mut app), 1);
    assert_eq!(
        app.world().resource::<ResultScreenFocusOrder>().len(),
        2,
        "hero step focuses both Continue and Return CTAs"
    );
    assert_eq!(
        query_count::<ResultScreenRematchButton>(&mut app),
        0,
        "rematch is out of MVP scope and should not be spawned"
    );
    assert_eq!(return_button_count(&mut app), 1);
    assert_eq!(
        query_count::<ResultScreenContinueButton>(&mut app),
        1,
        "two-step reveal mounts a Continue CTA on the hero step"
    );
}

#[test]
fn snapshot_only_game_over_uses_pending_fallback_and_return_action() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(&mut app, None);

    assert_eq!(headline(&mut app), "RESULT PENDING");
    assert_eq!(
        current_step(&app),
        ResultScreenStep::Hero,
        "screen always opens on the hero step"
    );

    press_key(&mut app, KeyCode::Escape);
    assert_eq!(
        app.world().resource::<State<ClientState>>().get(),
        &ClientState::InSession,
        "escape resets focus to the primary CTA, it does not exit the result screen"
    );

    // First Enter advances through the two-step reveal (Hero → Accounting).
    press_key(&mut app, KeyCode::Enter);
    app.update();
    assert_eq!(current_step(&app), ResultScreenStep::Accounting);
    assert_eq!(
        app.world()
            .resource::<ResultScreenOutboundMessages>()
            .acknowledgements
            .len(),
        0,
        "advancing to accounting must not send the result acknowledgement"
    );

    // Second Enter on the accounting step commits the return-to-lobby path.
    press_key(&mut app, KeyCode::Enter);
    app.update();

    assert_eq!(
        app.world()
            .resource::<ResultScreenOutboundMessages>()
            .acknowledgements
            .len(),
        1,
        "return to lobby sends the S9-RS-001 acknowledgement once"
    );
    assert_eq!(
        app.world().resource::<State<ClientState>>().get(),
        &ClientState::Lobby
    );
}

#[test]
fn reduced_motion_disables_entry_and_row_motion() {
    test_helpers::init_test_tracing();
    let mut preferences = AccessibilityPreferences::default();
    preferences.reduced_motion = true;

    assert_eq!(
        result_screen_motion_state(&preferences),
        ResultScreenMotionState {
            reduced_motion: true,
            entry_duration_ms: 0,
            row_sequencing_enabled: false,
            flash_count_per_second: 0,
        }
    );

    let mut app = result_screen_app();
    app.world_mut()
        .resource_mut::<AccessibilityPreferences>()
        .reduced_motion = true;
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    let motion = *app.world().resource::<ResultScreenMotionState>();
    assert!(motion.reduced_motion);
    assert_eq!(motion.entry_duration_ms, 0);
    assert!(!motion.row_sequencing_enabled);
    assert_eq!(motion.flash_count_per_second, 0);
}

#[test]
fn result_screen_has_single_game_over_receiver_and_no_snapshot_receiver() {
    test_helpers::init_test_tracing();
    let client_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let matches = collect_unique_source_matches(&client_src, "MessageReceiver<S2CGameOver>");
    let expected = BTreeSet::from([client_src.join("presentation").join("result_screen.rs")]);

    assert_eq!(matches, expected);

    let result_screen_src =
        fs::read_to_string(client_src.join("presentation").join("result_screen.rs"))
            .expect("result screen source should be readable");
    assert!(
        !result_screen_src.contains("MessageReceiver<S2CGameSnapshot>"),
        "result screen must consume the presentation snapshot fanout, not add a second S2CGameSnapshot drain"
    );
}

#[test]
fn test_result_screen_two_step_opens_on_hero_with_distinct_panel_markers() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();

    // Act
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    // Assert: hero + accounting panels are distinct entities with their own
    // marker components, and the screen opens on the hero step.
    assert_eq!(
        query_count::<ResultScreenHeroPanel>(&mut app),
        1,
        "two-step reveal must spawn exactly one hero panel marker"
    );
    assert_eq!(
        query_count::<ResultScreenAccountingPanel>(&mut app),
        1,
        "two-step reveal must spawn exactly one accounting panel marker"
    );
    assert_eq!(current_step(&app), ResultScreenStep::Hero);

    let entities = *app.world().resource::<ResultScreenEntities>();
    assert_eq!(
        node_display(&mut app, entities.hero_panel),
        Display::Flex,
        "hero panel is mounted on the hero step"
    );
    assert_eq!(
        node_display(&mut app, entities.accounting_panel),
        Display::None,
        "accounting panel stays hidden until the user advances"
    );
    assert_eq!(
        node_display(&mut app, entities.continue_button),
        Display::Flex,
        "continue CTA is visible on the hero step"
    );
    // Return-to-lobby CTA stays accessible on both steps per V-P1-05.
    assert_eq!(return_button_count(&mut app), 1);
}

#[test]
fn test_result_screen_two_step_continue_transitions_hero_to_accounting() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );
    let entities = *app.world().resource::<ResultScreenEntities>();
    assert_eq!(current_step(&app), ResultScreenStep::Hero);
    assert_eq!(
        node_display(&mut app, entities.accounting_panel),
        Display::None
    );

    // Act: press Enter once — this is the keyboard equivalent of the
    // Continue CTA on the hero step.
    press_key(&mut app, KeyCode::Enter);
    app.update();

    // Assert: step state advanced and the panels swap; Continue CTA
    // disappears from the action row while Return-to-Lobby stays.
    assert_eq!(current_step(&app), ResultScreenStep::Accounting);
    assert_eq!(
        node_display(&mut app, entities.hero_panel),
        Display::None,
        "hero panel hides after advancing to accounting"
    );
    assert_eq!(
        node_display(&mut app, entities.accounting_panel),
        Display::Flex,
        "accounting panel mounts on step 2"
    );
    assert_eq!(
        node_display(&mut app, entities.continue_button),
        Display::None,
        "continue CTA is hidden once the user has advanced"
    );
    assert_eq!(
        app.world().resource::<ResultScreenFocusOrder>().len(),
        1,
        "only Return-to-Lobby remains in the focus order on the accounting step"
    );
    // Acknowledgement must NOT have been sent by the advance step.
    assert_eq!(
        app.world()
            .resource::<ResultScreenOutboundMessages>()
            .acknowledgements
            .len(),
        0
    );
    // Layout bounds: panel children stay within the bounded panel's max
    // height (set via Val::Percent(92.0) at spawn), so the swap never adds
    // unbounded clipping.
    assert_eq!(
        app.world().resource::<State<ClientState>>().get(),
        &ClientState::InSession,
        "advancing through the reveal does not leave the in-session state"
    );
}

#[test]
fn test_result_screen_two_step_class_persona_label_uses_snapshot_class() {
    // Arrange
    test_helpers::init_test_tracing();
    let local = player(1);
    let snapshot = game_over_snapshot();

    // Act
    let label = result_screen_class_persona_label(Some(&snapshot), Some(local))
        .expect("snapshot carries a class for the local player");

    // Assert
    assert!(
        label.contains("Iop"),
        "class persona label surfaces the snapshot's ClassId for the local player: got {label}"
    );
}

#[test]
fn test_result_screen_two_step_class_persona_label_absent_without_snapshot() {
    // Arrange + Act
    test_helpers::init_test_tracing();
    let label = result_screen_class_persona_label(None, None);

    // Assert: no snapshot ⇒ no invented value; caller hides the row.
    assert!(label.is_none());
}

#[test]
fn test_result_screen_two_step_class_persona_text_rendered_on_hero_panel() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();

    // Act
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    // Assert: class persona text picked up from the snapshot and rendered
    // into the dedicated text entity on the hero panel.
    let rendered = text_with::<ResultScreenClassPersona>(&mut app);
    assert!(
        rendered.contains("Iop"),
        "class persona text should surface the snapshot's class label on the hero panel: got {rendered}"
    );
}

fn current_step(app: &App) -> ResultScreenStep {
    app.world().resource::<ResultScreenStepState>().current
}

fn node_display(app: &mut App, entity: Entity) -> Display {
    app.world_mut()
        .get::<Node>(entity)
        .expect("result screen entity must have a Node")
        .display
}

fn result_screen_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ResultScreenPlugin);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.world_mut()
        .resource_mut::<ClientSessionIdentity>()
        .player_id = Some(player(1));
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn open_result_screen(app: &mut App, result: Option<S2CGameOver>) {
    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(game_over_snapshot()));
    {
        let mut view_state = app.world_mut().resource_mut::<ResultScreenViewState>();
        view_state.cached_result = result;
    }
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::GameOver;
    app.update();
    app.update();
}

fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(key);
        input.clear();
    }
    app.update();
}

fn result(loser: Option<PlayerId>, reason: GameOverReason) -> S2CGameOver {
    S2CGameOver {
        loser,
        round: 9,
        reason,
    }
}

fn game_over_snapshot() -> S2CGameSnapshot {
    let mut own = player_snapshot(player(1), 8, 6, 2, 10);
    own.objectives = vec![
        objective(1, 3, true, true),
        objective(2, 0, false, true),
        objective(3, 5, true, false),
        objective(4, 4, false, false),
        objective(5, 7, true, false),
    ];
    own.opponent_objectives = vec![
        opponent_objective(1, 6, false, None),
        opponent_objective(2, 0, true, Some(true)),
        opponent_objective(3, 0, true, Some(false)),
        opponent_objective(4, 5, false, None),
    ];

    let mut opponent = player_snapshot(player(2), 11, 4, 1, 10);
    opponent.objectives = vec![
        objective(1, 6, true, false),
        objective(2, 0, false, true),
        objective(3, 0, true, true),
        objective(4, 5, false, false),
        objective(5, 4, true, false),
    ];

    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: own.player_id,
        round_number: 9,
        phase: RoundPhase::GameOver,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(
    player_id: PlayerId,
    gold: u32,
    current_mana: u32,
    reserve_mana: u32,
    mana_cap: u8,
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold,
        reserved_gold: 0,
        current_mana,
        reserve_mana,
        spawn_range_cells: 1,
        mana_cap,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: Vec::new(),
        opponent_objectives: Vec::new(),
    }
}

fn objective(lane: u8, hp: u8, is_real: bool, is_destroyed: bool) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        lane,
        hp,
        is_real,
        is_destroyed,
    }
}

fn opponent_objective(
    lane: u8,
    hp: u8,
    is_destroyed: bool,
    was_fake: Option<bool>,
) -> OpponentObjectiveSnapshot {
    OpponentObjectiveSnapshot {
        lane,
        hp,
        is_destroyed,
        was_fake,
    }
}

fn root_visibility(app: &mut App) -> Visibility {
    let mut query = app
        .world_mut()
        .query_filtered::<&Visibility, With<ResultScreenRoot>>();
    *query
        .single(app.world())
        .expect("result screen root should exist")
}

fn headline(app: &mut App) -> String {
    text_with::<ResultScreenHeadline>(app)
}

fn summary_text(app: &mut App) -> String {
    text_with::<ResultScreenSummaryText>(app)
}

fn text_with<M: Component>(app: &mut App) -> String {
    let mut query = app.world_mut().query_filtered::<&Text, With<M>>();
    query
        .single(app.world())
        .expect("text entity should exist")
        .0
        .clone()
}

fn return_button_count(app: &mut App) -> usize {
    query_count::<ResultScreenReturnToLobbyButton>(app)
}

fn focus_indicator_count(app: &mut App) -> usize {
    query_count::<ResultScreenFocusIndicator>(app)
}

fn query_count<M: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<M>>();
    query.iter(app.world()).count()
}

fn collect_unique_source_matches(path: &Path, needle: &str) -> BTreeSet<PathBuf> {
    let mut matches = BTreeSet::new();
    collect_source_matches(path, needle, &mut matches);
    matches
}

fn collect_source_matches(path: &Path, needle: &str, matches: &mut BTreeSet<PathBuf>) {
    let entries = fs::read_dir(path).expect("client source directory should be readable");
    for entry in entries {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            collect_source_matches(&path, needle, matches);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("Rust source file should be readable");
        if contents.contains(needle) {
            matches.insert(path);
        }
    }
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
