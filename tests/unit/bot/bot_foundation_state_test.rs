//! Foundation-slice tests for the bot state scaffold (PROMPT 1428).
//!
//! Scope: prove the new resources are insertable, the deterministic seed
//! reproduces, and the decision log appends correctly. No system behaviour
//! is exercised — that lands with later phase prompts.

use bevy::prelude::*;
use rand::RngCore;
use server::feature::bot::{
    BotDecisionEntry, BotDecisionKind, BotDecisionLog, BotDifficulty, BotPlayers, BotState,
    BotThinkDelayWindow, BOT_AUCTION_PASS_THRESHOLD_MS, BOT_SAFETY_MARGIN_MS,
    BOT_THINK_DELAY_MAX_MS, BOT_THINK_DELAY_MIN_MS,
};
use shared::card::ClassId;
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

#[test]
fn test_bot_foundation_resources_are_insertable_into_fresh_app() {
    // Arrange
    let mut app = App::new();

    // Act
    app.init_resource::<BotPlayers>();
    app.init_resource::<BotDecisionLog>();

    // Assert
    assert!(app.world().contains_resource::<BotPlayers>());
    assert!(app.world().contains_resource::<BotDecisionLog>());
    assert!(app.world().resource::<BotPlayers>().is_empty());
    assert!(app.world().resource::<BotDecisionLog>().is_empty());
}

#[test]
fn test_bot_state_new_initialises_defaults_for_mvp_slice() {
    // Arrange
    let player = PlayerId(7);
    let seed: u64 = 0xDEAD_BEEF_CAFE_F00D;

    // Act
    let state = BotState::new(player, seed);

    // Assert
    assert_eq!(state.player_id, player);
    assert_eq!(state.difficulty, BotDifficulty::Mvp);
    assert_eq!(state.rng_seed, seed);
    assert_eq!(state.rng_word_counter, 0);
    assert_eq!(state.last_decision_at_ms, None);
    assert_eq!(state.think_delay, BotThinkDelayWindow::default());
    assert_eq!(state.think_delay.min_ms, BOT_THINK_DELAY_MIN_MS);
    assert_eq!(state.think_delay.max_ms, BOT_THINK_DELAY_MAX_MS);
    assert_eq!(state.phase_timing.next_decision_at_ms, None);
    assert_eq!(state.phase_timing.failsafe_deadline_ms, None);
    assert_eq!(state.class_choice, None);
}

#[test]
fn test_bot_state_rng_is_deterministic_across_two_instances() {
    // Arrange
    let player = PlayerId(11);
    let seed: u64 = 0x0123_4567_89AB_CDEF;
    let mut left = BotState::new(player, seed);
    let mut right = BotState::new(player, seed);

    // Act
    let left_words: [u64; 4] = [
        left.rng.next_u64(),
        left.rng.next_u64(),
        left.rng.next_u64(),
        left.rng.next_u64(),
    ];
    let right_words: [u64; 4] = [
        right.rng.next_u64(),
        right.rng.next_u64(),
        right.rng.next_u64(),
        right.rng.next_u64(),
    ];

    // Assert
    assert_eq!(left_words, right_words);
}

#[test]
fn test_bot_state_rng_diverges_for_different_seeds() {
    // Arrange
    let player = PlayerId(11);
    let mut bot_a = BotState::new(player, 1);
    let mut bot_b = BotState::new(player, 2);

    // Act
    let word_a = bot_a.rng.next_u64();
    let word_b = bot_b.rng.next_u64();

    // Assert
    assert_ne!(
        word_a, word_b,
        "different seeds must yield different first RNG draws"
    );
}

#[test]
fn test_bot_players_insert_then_get_round_trips_state_by_player_id() {
    // Arrange
    let player = PlayerId(3);
    let mut bots = BotPlayers::default();

    // Act
    bots.insert(BotState::new(player, 42));

    // Assert
    assert!(bots.contains(player));
    assert_eq!(bots.len(), 1);
    let stored = bots.get(player).expect("bot state present");
    assert_eq!(stored.player_id, player);
    assert_eq!(stored.rng_seed, 42);
}

#[test]
fn test_bot_decision_log_push_appends_in_order() {
    // Arrange
    let mut log = BotDecisionLog::default();
    let bot = PlayerId(5);
    let entry_one = BotDecisionEntry {
        round_number: 0,
        phase: RoundPhase::Lobby,
        bot_player_id: bot,
        decision: BotDecisionKind::ClassChosen {
            class_id: ClassId::Iop,
        },
        timestamp_ms: 100,
        legal_action_count: Some(6),
        seed: 42,
        seed_word_counter: 1,
    };
    let entry_two = BotDecisionEntry {
        round_number: 1,
        phase: RoundPhase::DraftInitial,
        bot_player_id: bot,
        decision: BotDecisionKind::DraftReady,
        timestamp_ms: 1_500,
        legal_action_count: None,
        seed: 42,
        seed_word_counter: 1,
    };

    // Act
    log.push(entry_one.clone());
    log.push(entry_two.clone());

    // Assert
    assert_eq!(log.len(), 2);
    assert_eq!(log.entries[0], entry_one);
    assert_eq!(log.entries[1], entry_two);
    assert_eq!(log.last(), Some(&entry_two));
}

#[test]
fn test_bot_decision_log_clear_resets_entries_but_keeps_resource() {
    // Arrange
    let mut log = BotDecisionLog::default();
    log.push(BotDecisionEntry {
        round_number: 0,
        phase: RoundPhase::Lobby,
        bot_player_id: PlayerId(1),
        decision: BotDecisionKind::ClassConfirmed,
        timestamp_ms: 0,
        legal_action_count: None,
        seed: 0,
        seed_word_counter: 0,
    });

    // Act
    log.clear();

    // Assert
    assert!(log.is_empty());
    assert_eq!(log.len(), 0);
    assert_eq!(log.last(), None);
}

#[test]
fn test_bot_timing_constants_match_audit_contract() {
    // Arrange / Act / Assert — wired constants against PROMPT-1423 §3.3
    assert_eq!(BOT_THINK_DELAY_MIN_MS, 300);
    assert_eq!(BOT_THINK_DELAY_MAX_MS, 1_200);
    assert_eq!(BOT_SAFETY_MARGIN_MS, 800);
    assert_eq!(BOT_AUCTION_PASS_THRESHOLD_MS, 500);
    assert!(BOT_THINK_DELAY_MIN_MS < BOT_THINK_DELAY_MAX_MS);
}
