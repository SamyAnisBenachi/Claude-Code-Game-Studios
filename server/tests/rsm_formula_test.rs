// RSM Formula Tests — round-state-machine.md Formula F1
// ADR-009: RoundPhase uses plain Resource; advance_phase is sole writer
// Validation criteria: RSM-3 (R=3 → true), RSM-4 (R=4 → false), RSM-5 (R=9 → true)
// Run: cargo test -p server

/// Formula F1 from round-state-machine.md.
/// is_auction_round(R) = (R mod 3 == 0)
/// Invariant: round_number >= 1 must hold before calling (see ADR-009 §8).
fn is_auction_round(round_number: u32) -> bool {
    debug_assert!(round_number >= 1, "round_number was 0 — impossible");
    round_number % 3 == 0
}

#[test]
fn test_round_1_is_not_auction_round() {
    assert!(!is_auction_round(1));
}

#[test]
fn test_round_2_is_not_auction_round() {
    assert!(!is_auction_round(2));
}

#[test]
fn test_round_3_is_auction_round() {
    // RSM-3: first auction occurs at round 3
    assert!(is_auction_round(3));
}

#[test]
fn test_round_4_is_not_auction_round() {
    // RSM-4
    assert!(!is_auction_round(4));
}

#[test]
fn test_round_6_is_auction_round() {
    assert!(is_auction_round(6));
}

#[test]
fn test_round_9_is_auction_round() {
    // RSM-5
    assert!(is_auction_round(9));
}

#[test]
fn test_auction_density_one_in_three_rounds() {
    // rounds 1-12: exactly 4 auction rounds (3, 6, 9, 12)
    let auction_rounds: Vec<u32> = (1..=12).filter(|&r| is_auction_round(r)).collect();
    assert_eq!(auction_rounds, vec![3, 6, 9, 12]);
}
