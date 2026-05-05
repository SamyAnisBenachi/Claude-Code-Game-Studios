# NP-005 Placement Payload Shape Split Evidence

Story: `production/epics/lightyear-protocol-verification/story-005-placement-payload-shape-split.md`
Requirement: `TR-NP-013`
Date: 2026-05-05

## Verification Commands

- `cargo fmt -p shared -- --check` - PASS
- `cargo fmt -p server -- --check` - PASS
- `cargo fmt -p client -- --check` - PASS
- `cargo check -p shared` - PASS
- `cargo test -p shared` - PASS; 5 passed
- `cargo check -p server` - PASS
- `cargo check -p client` - PASS

## Shape Evidence

Command:

```powershell
rg -n "placements: Vec<PlacedCardSubmit>|placements: Vec<PlacedCardReveal>|register_c2s::<C2SSubmitPlacement>|register_s2c::<S2CPlacementReveal>" shared/src/protocol.rs
```

Output:

```text
67:    register_c2s::<C2SSubmitPlacement>(registry, ProtocolChannel::Reliable);
83:    register_s2c::<S2CPlacementReveal>(registry, ProtocolChannel::Reliable);
345:    pub placements: Vec<PlacedCardSubmit>,
430:    pub placements: Vec<PlacedCardReveal>,
```

Command:

```powershell
rg -n "pub struct PlacedCard$|Vec<PlacedCard>|S2CPlacementReveal \{ placements: Vec<PlacedCard>|C2SSubmitPlacement \{ placements: Vec<PlacedCard>" shared/src server/src client/src
```

Output:

```text
No matches.
```

Command:

```powershell
rg -n "AcceptedPlacement|PlacementSubmissionReceived|S2CPlacementReveal|PlacedCardReveal|PlacedCardSubmit" server/src/feature/board/placement.rs server/src/feature/combat/mod.rs client/src/ui/hand/mod.rs
```

Output excerpt:

```text
client/src/ui/hand/mod.rs:169:    pub placements: Vec<PlacedCardSubmit>,
server/src/feature/board/placement.rs:56:    pub placements: Vec<PlacedCardSubmit>,
server/src/feature/board/placement.rs:78:pub struct AcceptedPlacement {
server/src/feature/board/placement.rs:97:    pub fn reveal(&self) -> PlacedCardReveal {
server/src/feature/board/placement.rs:436:    let reveal = S2CPlacementReveal {
server/src/feature/combat/mod.rs:1700:            placements: placements.iter().map(AcceptedPlacement::reveal).collect(),
```

## Privacy Check

`PlacedCardReveal` contains only `owner_id`, `card_id`, and `target`. The shared
protocol unit test `submit_and_reveal_payloads_use_direction_specific_shapes`
serializes `S2CPlacementReveal` and asserts that `current_mana_spend`,
`reserve_mana_spend`, and legacy `reserve_amount` are absent from reveal entries.
