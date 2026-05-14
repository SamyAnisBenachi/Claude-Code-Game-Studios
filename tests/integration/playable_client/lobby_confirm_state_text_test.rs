//! Lobby confirm-state text differentiation test.
//!
//! Story: `S11-LOBBY-UX-CONFIRM-STATE-001` (story-023). Asserts that the
//! lobby confirm-button text renders the State A variant ("waiting for
//! opponent's confirm") and the State B variant ("local player has not
//! confirmed yet") as two distinct strings, and that the differentiation
//! is preserved across the Sprint 12 story 013 duplicate-confirm
//! fallback path (AC6).
//!
//! No client-side class-lock authority is introduced: state transitions
//! are driven exclusively by `S2CClassLocked` and `S2CClassesRevealed`
//! reads via `apply_class_locked` / `apply_classes_revealed` (ADR-002
//! binding reinforced by Sprint 12 story 013 at `d8d0196`).

use client::ui::lobby::{
    apply_class_locked, apply_classes_revealed, lobby_confirm_button_text, LobbyInputState,
    LobbyViewState,
};
use shared::card::ClassId;
use shared::protocol::{S2CClassLocked, S2CClassesRevealed};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const STATE_A_TEXT: &str = "Waiting for opponent...";
const STATE_B_TEXT: &str = "Confirm your class to continue";

#[test]
fn state_b_renders_when_local_player_has_not_confirmed() {
    test_helpers::init_test_tracing();
    let lobby = LobbyViewState::default();
    let input = LobbyInputState::default();

    let text = lobby_confirm_button_text(&lobby, &input);

    assert_eq!(text, STATE_B_TEXT);
}

#[test]
fn state_a_renders_after_own_class_locked_before_opponent_reveal() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    let input = LobbyInputState::default();

    apply_class_locked(
        &mut lobby,
        &S2CClassLocked {
            class_id: ClassId::Iop,
        },
    );

    let text = lobby_confirm_button_text(&lobby, &input);

    assert_eq!(text, STATE_A_TEXT);
    assert!(lobby.revealed_classes.is_empty());
}

#[test]
fn state_a_and_state_b_are_distinct_strings() {
    test_helpers::init_test_tracing();
    let input = LobbyInputState::default();

    let state_b = lobby_confirm_button_text(&LobbyViewState::default(), &input);

    let mut state_a_lobby = LobbyViewState::default();
    apply_class_locked(
        &mut state_a_lobby,
        &S2CClassLocked {
            class_id: ClassId::Sacrier,
        },
    );
    let state_a = lobby_confirm_button_text(&state_a_lobby, &input);

    assert_ne!(state_a, state_b);
}

#[test]
fn ac6_duplicate_confirm_reack_keeps_state_a_variant() {
    // Sprint 12 story 013 fallback path (`d8d0196`): the server returns an
    // `S2CClassLocked` re-ack on a duplicate same-class confirm. The
    // re-ack must land the local player in State A, not flip them back to
    // State B.
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    let input = LobbyInputState::default();

    apply_class_locked(
        &mut lobby,
        &S2CClassLocked {
            class_id: ClassId::Iop,
        },
    );
    apply_class_locked(
        &mut lobby,
        &S2CClassLocked {
            class_id: ClassId::Iop,
        },
    );

    let text = lobby_confirm_button_text(&lobby, &input);

    assert_eq!(text, STATE_A_TEXT);
}

#[test]
fn post_reveal_text_is_distinct_from_state_a_and_state_b() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    let input = LobbyInputState::default();

    let state_b = lobby_confirm_button_text(&lobby, &input);

    apply_class_locked(
        &mut lobby,
        &S2CClassLocked {
            class_id: ClassId::Cra,
        },
    );
    let state_a = lobby_confirm_button_text(&lobby, &input);

    apply_classes_revealed(
        &mut lobby,
        &S2CClassesRevealed {
            player_class_map: vec![(PlayerId(1), ClassId::Cra), (PlayerId(2), ClassId::Xelor)],
        },
    );
    let post_reveal = lobby_confirm_button_text(&lobby, &input);

    assert_ne!(state_a, state_b);
    assert_ne!(state_a, post_reveal);
    assert_ne!(state_b, post_reveal);
}
