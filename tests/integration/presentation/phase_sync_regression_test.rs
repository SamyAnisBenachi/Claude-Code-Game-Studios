//! Regression tests for BUG-01 / BUG-13 (PROMPT-2025): client phase_label and
//! client_state_label stay at Lobby/0 for the entire game while the server
//! advances through all phases.
//!
//! Root cause chain (documented in reports/PROMPT-2030-client-phase-sync-p0-repair.md):
//!
//! 1. `phase_sink_system` reads from `MessageReceiver<S2CPhaseChanged>` (a
//!    Lightyear component) and calls `apply_phase_changed_message` to update
//!    `CurrentClientPhase`.
//! 2. `autoplay.rs::publish_status_system` and `qa_snapshot.rs::write_qa_snapshot_system`
//!    both export `phase_label`/`round` directly from `Res<CurrentClientPhase>`.
//! 3. If `MessageReceiver<S2CPhaseChanged>` is never populated (server never
//!    sends the message, or the game session never starts), `CurrentClientPhase`
//!    retains its default `{ phase: Lobby, round: 0 }` for the entire run.
//!
//! These tests exercise the pure-function layers that the system depends on,
//! proving that the client-side code path is correct.  They would catch any
//! regression that broke `apply_phase_changed_message`,
//! `should_enter_session_from_phase`, or the `apply_phase_changed_messages`
//! wrapper — the three surfaces a future engineer might touch that would
//! silently re-introduce BUG-01.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    presentation::{apply_phase_changed_messages, CurrentClientPhase, PresentationPlugin},
    state::{
        apply_phase_changed_message, should_enter_session_from_phase, ClientSessionIdentity,
        ClientState,
    },
};
use shared::{
    protocol::{RoundPhase, S2CPhaseChanged},
    session::PlayerId,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ---------------------------------------------------------------------------
// apply_phase_changed_message — pure function regression
// ---------------------------------------------------------------------------

/// Applying a non-Lobby phase to the default CurrentClientPhase must update
/// both `phase` and `round`.  This is the innermost function called by
/// `phase_sink_system`; if it breaks, the client can never leave Lobby/0.
#[test]
fn test_apply_phase_changed_message_draft_initial_leaves_lobby() {
    let mut current = CurrentClientPhase::default();
    assert_eq!(current.phase, RoundPhase::Lobby, "precondition: default is Lobby");
    assert_eq!(current.round, 0, "precondition: default round is 0");

    apply_phase_changed_message(
        S2CPhaseChanged {
            phase: RoundPhase::DraftInitial,
            round_number: 1,
            timer_duration_ms: 44_999,
        },
        &mut current,
    );

    assert_ne!(
        current.phase,
        RoundPhase::Lobby,
        "phase must leave Lobby after S2CPhaseChanged(DraftInitial)"
    );
    assert_eq!(current.phase, RoundPhase::DraftInitial);
    assert_eq!(current.round, 1);
}

/// BUG-01 regression: `phase_label` reads `format!("{:?}", current.phase)`.
/// After the message is applied, the formatted label must not be "Lobby".
#[test]
fn test_phase_label_is_not_lobby_after_draft_initial_applied() {
    let mut current = CurrentClientPhase::default();

    apply_phase_changed_message(
        S2CPhaseChanged {
            phase: RoundPhase::DraftInitial,
            round_number: 1,
            timer_duration_ms: 44_999,
        },
        &mut current,
    );

    let phase_label = format!("{:?}", current.phase);
    assert_ne!(
        phase_label, "Lobby",
        "phase_label must not be \"Lobby\" after S2CPhaseChanged(DraftInitial) is applied"
    );
    assert_eq!(phase_label, "DraftInitial");
}

/// Verify all non-Lobby phases leave RoundPhase::Lobby.
#[test]
fn test_apply_phase_changed_message_all_game_phases_leave_lobby() {
    let game_phases = [
        RoundPhase::DraftInitial,
        RoundPhase::DraftAuction,
        RoundPhase::DraftShop,
        RoundPhase::Placement,
        RoundPhase::Resolution,
        RoundPhase::GameOver,
    ];

    for phase in game_phases {
        let mut current = CurrentClientPhase::default();
        apply_phase_changed_message(
            S2CPhaseChanged {
                phase,
                round_number: 1,
                timer_duration_ms: 0,
            },
            &mut current,
        );
        assert_ne!(
            current.phase,
            RoundPhase::Lobby,
            "phase {:?} must leave Lobby after apply_phase_changed_message",
            phase
        );
        assert_eq!(current.phase, phase);
    }
}

/// Last-write-wins across multiple messages — same contract as the existing
/// `phase_sink_application_is_last_write_wins_and_ignores_timer_data` test
/// but anchored to Lobby/0 as the starting point to match the BUG-01 scenario.
#[test]
fn test_apply_phase_changed_messages_last_write_wins_from_lobby() {
    let mut current = CurrentClientPhase::default();

    apply_phase_changed_messages(
        [
            S2CPhaseChanged {
                phase: RoundPhase::DraftInitial,
                round_number: 1,
                timer_duration_ms: 44_999,
            },
            S2CPhaseChanged {
                phase: RoundPhase::Placement,
                round_number: 1,
                timer_duration_ms: 10_000,
            },
        ],
        &mut current,
    );

    assert_eq!(current.phase, RoundPhase::Placement);
    assert_eq!(current.round, 1);
}

// ---------------------------------------------------------------------------
// should_enter_session_from_phase — session-gate regression (BUG-13 path)
// ---------------------------------------------------------------------------

/// BUG-13 regression: `ClientState` transitions to `InSession` only when
/// `should_enter_session_from_phase` returns true.  If this gate accidentally
/// returns false for DraftInitial, `ClientState` stays Lobby forever.
#[test]
fn test_should_enter_session_from_draft_initial_with_player_id() {
    let identity = ClientSessionIdentity {
        player_id: Some(PlayerId(9)),
        session_id: Some(9),
        session_token: Some([0u8; 16]),
    };

    assert!(
        should_enter_session_from_phase(&identity, RoundPhase::DraftInitial),
        "DraftInitial with player_id set must return true"
    );
}

#[test]
fn test_should_not_enter_session_for_lobby_phase_even_with_player_id() {
    let identity = ClientSessionIdentity {
        player_id: Some(PlayerId(1)),
        session_id: Some(1),
        session_token: None,
    };

    assert!(
        !should_enter_session_from_phase(&identity, RoundPhase::Lobby),
        "Lobby phase must NOT trigger InSession transition"
    );
}

#[test]
fn test_should_not_enter_session_without_player_id() {
    let identity = ClientSessionIdentity::default();

    assert!(
        !should_enter_session_from_phase(&identity, RoundPhase::DraftInitial),
        "DraftInitial without player_id must NOT trigger InSession transition"
    );
}

#[test]
fn test_should_enter_session_for_all_in_game_phases_with_player_id() {
    let identity = ClientSessionIdentity {
        player_id: Some(PlayerId(1)),
        session_id: Some(1),
        session_token: None,
    };

    let in_game_phases = [
        RoundPhase::DraftInitial,
        RoundPhase::DraftAuction,
        RoundPhase::DraftShop,
        RoundPhase::Placement,
        RoundPhase::Resolution,
        RoundPhase::GameOver,
    ];

    for phase in in_game_phases {
        assert!(
            should_enter_session_from_phase(&identity, phase),
            "phase {:?} with player_id must return true",
            phase
        );
    }
}

// ---------------------------------------------------------------------------
// CurrentClientPhase resource lifetime — integration regression
// ---------------------------------------------------------------------------

/// PresentationPlugin must initialize CurrentClientPhase so the autoplay
/// `phase_label` export always has a resource to read.  If it's absent,
/// `Option<Res<CurrentClientPhase>>` returns None and phase_label is null
/// instead of "Lobby".
#[test]
fn test_presentation_plugin_initializes_current_client_phase() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.add_plugins(PresentationPlugin);
    app.update();

    let current = app.world().get_resource::<CurrentClientPhase>();
    assert!(
        current.is_some(),
        "PresentationPlugin must initialize CurrentClientPhase"
    );
    let current = current.unwrap();
    assert_eq!(
        current.phase,
        RoundPhase::Lobby,
        "default phase must be Lobby"
    );
    assert_eq!(current.round, 0, "default round must be 0");
}

/// Direct mutation of CurrentClientPhase (as done by apply_phase_changed_message)
/// is observable via Res<CurrentClientPhase> in the same frame — proves the
/// resource is not shadowed or double-initialized.
#[test]
fn test_current_client_phase_mutation_is_observable_from_resource() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.add_plugins(PresentationPlugin);
    app.update();

    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        apply_phase_changed_message(
            S2CPhaseChanged {
                phase: RoundPhase::Placement,
                round_number: 2,
                timer_duration_ms: 10_000,
            },
            &mut current,
        );
    }

    let current = app.world().resource::<CurrentClientPhase>();
    assert_eq!(current.phase, RoundPhase::Placement);
    assert_eq!(current.round, 2);
}

/// ClientState starts as Lobby and its label must match "Lobby" — mirrors the
/// BUG-13 observation that `client_state_label` was always "Lobby".
#[test]
fn test_client_state_default_is_lobby_and_label_matches() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.add_plugins(PresentationPlugin);
    app.update();

    let state = app.world().get_resource::<State<ClientState>>().unwrap();
    let label = format!("{:?}", state.get());
    assert_eq!(label, "Lobby", "ClientState default label must be \"Lobby\"");

    // Transition to InSession and verify label changes — this is what the
    // fix must enable for in-game snapshots.
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    let state = app.world().get_resource::<State<ClientState>>().unwrap();
    let label = format!("{:?}", state.get());
    assert_eq!(
        label, "InSession",
        "ClientState label must be \"InSession\" after transition"
    );
}
