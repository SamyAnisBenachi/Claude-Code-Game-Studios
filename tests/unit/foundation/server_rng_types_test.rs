// Tests for ServerRng type definitions and audit infrastructure.
// Story 001: server-rng epic, Foundation layer.
// Required evidence: tests/unit/foundation/server_rng_types_test.rs (Logic story — BLOCKING)

#[cfg(test)]
mod tests {
    use server::foundation::rng::{RngEvent, ServerRng};

    // RNG1: After ServerRng::new(), current_seed_index() == 1
    #[test]
    fn test_new_seed_index_is_one() {
        let rng = ServerRng::from_seed(0);
        assert_eq!(
            rng.current_seed_index(),
            1,
            "index 0 is consumed by SessionInit sentinel; gameplay starts at 1"
        );
    }

    // RNG5: 0 gameplay calls → 1 audit entry (sentinel only)
    #[test]
    fn test_zero_calls_has_one_audit_entry() {
        let rng = ServerRng::from_seed(42);
        assert_eq!(
            rng.audit_log().len(),
            1,
            "only the SessionInit sentinel should be present"
        );
    }

    // RNG5: N gameplay calls → N+1 audit entries
    #[test]
    fn test_n_calls_produces_n_plus_one_audit_entries() {
        let mut rng = ServerRng::from_seed(42);
        rng.next_seed(RngEvent::ResolveEcaflip { lane: 0 }, None);
        rng.next_seed(RngEvent::ResolveEcaflip { lane: 1 }, None);
        rng.next_seed(RngEvent::ResolveEcaflip { lane: 2 }, None);
        assert_eq!(
            rng.audit_log().len(),
            4,
            "1 sentinel + 3 gameplay calls = 4 entries"
        );
        assert_eq!(rng.current_seed_index(), 4);
    }

    // RNG11: audit_log()[0] is SessionInit with result = None
    #[test]
    fn test_sentinel_is_session_init_with_no_result() {
        let rng = ServerRng::from_seed(12345);
        let first = &rng.audit_log()[0];
        assert_eq!(first.event_type, RngEvent::SessionInit);
        assert_eq!(first.seed_index, 0);
        assert!(first.result.is_none(), "SessionInit must have result = None");
    }

    // RNG11: no raw seed bytes appear in any AuditEntry.result
    #[test]
    fn test_no_raw_seed_in_audit_log() {
        let seed: u64 = 12345;
        let mut rng = ServerRng::from_seed(seed);
        rng.next_seed(RngEvent::ResolveEcaflip { lane: 0 }, Some("4".to_string()));
        for entry in rng.audit_log() {
            if let Some(result) = &entry.result {
                assert!(
                    !result.contains(&seed.to_string()),
                    "raw seed value must not appear in any audit entry result"
                );
            }
        }
    }

    // seed_index values in audit log are monotonically ordered 0..N
    #[test]
    fn test_audit_log_seed_indices_are_sequential() {
        let mut rng = ServerRng::from_seed(7);
        rng.next_seed(RngEvent::DrawFreeCard { player_id: 1 }, None);
        rng.next_seed(RngEvent::DrawFreeCard { player_id: 2 }, None);
        let log = rng.audit_log();
        for (i, entry) in log.iter().enumerate() {
            assert_eq!(
                entry.seed_index,
                i as u32,
                "seed_index at position {} should be {}",
                i,
                i
            );
        }
    }
}
