# S8-QA-001 Client B Log

**Client role**: Joiner / Client B  
**Commit**: `3cc620cdeee6f5249e404703365b160ccbc34f6c`  
**Evidence source**:
`production/qa/evidence/captures/sprint-8-friend-game-loop/playable-004-result-endpoint-trace.json`

Full browser/native Client B GUI execution was not completed in this Codex
session. The blocker is recorded in
`s8-qa-001-manual-smoke-summary.json`: this tool surface can run commands and
inspect artifacts, but it cannot interactively drive two external Bevy client
windows or browser clients through the full route. The nearest covered Client B
evidence is the controlled real-Lightyear joiner route below.

Observed joiner protocol route:

```text
C2SHello
S2CHandshake
C2SJoinRoom(room_code=<ROOM_CODE_REDACTED>)
S2CJoinAck
C2SSelectClass(Cra)
C2SConfirmClass(Cra)
S2CClassLocked
S2CClassesRevealed
S2CPhaseChanged(DraftInitial)
S2CDraftOffering(card_count=9)
C2SPurchaseCard(card_id=<SERVER_OFFERED_CARD_ID>)
S2CCardAcquired(source=DraftInitial)
S2CGoldUpdate(after_purchase=true)
C2SSignalReady(retract=false)
S2CPhaseChanged(Placement)
C2SSubmitPlacement(placements=[])
S2CPlacementReveal(placements=[])
S2CPhaseChanged(Resolution)
S2CResolutionEvent(events=[SubStepBegin])
S2CPhaseChanged(DraftShop)
C2SSignalReady(retract=false)
S2CPhaseChanged(Placement)
C2SSubmitPlacement(placements=[<SERVER_HAND_CARD>])
S2CPlacementReveal(placements=[<SERVER_ACCEPTED_CARD>])
S2CPhaseChanged(Resolution)
S2CResolutionEvent(events=[SubStepBegin,UnitPlaced])
S2CPhaseChanged(DraftAuction)
S2CAuctionCard(card_id=<SERVER_AUCTION_CARD>, starting_price=<SERVER_PRICE>)
S2CAuctionBidAccepted
S2CAuctionSettled(winner=host)
S2CPhaseChanged(DraftShop)
C2SSignalReady(retract=false)
S2CPhaseChanged(Placement)
C2SSubmitPlacement(placements=[<SERVER_HAND_CARD>])
S2CPlacementReveal(placements=[<SERVER_ACCEPTED_CARD>])
S2CPhaseChanged(Resolution)
S2CResolutionEvent(events=[SubStepBegin,UnitPlaced,ObjectiveDestroyed])
S2CPhaseChanged(DraftShop)
C2SSignalReady(retract=false)
S2CPhaseChanged(Placement)
C2SSubmitPlacement(placements=[])
S2CPlacementReveal(placements=[])
S2CPhaseChanged(Resolution)
S2CResolutionEvent(events=[ObjectiveDestroyed])
S2CGameOver(reason=Draw, loser=None)
S2CPhaseChanged(GameOver)
```

Manual browser game-over is not claimed. Automated same-commit real-Lightyear
GAME_OVER endpoint evidence is retained.
