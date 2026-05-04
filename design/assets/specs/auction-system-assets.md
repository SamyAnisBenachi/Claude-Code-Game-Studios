# Asset Specs - System: Auction System

> **Source**: design/gdd/auction-system.md; design/gdd/shop-auction-ui.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-04
> **Status**: 10 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-175 through ASSET-184

---

## Scope Notes

Shop / Auction UI already owns the shared auction panel chrome, gold icon, rarity gems, bid pulse ring, and draft/shop audio cues. This spec adds the Auction System's missing ownership: auction-specific audio, timer material behavior, local-expiry feedback, and disconnect grace UI.

**Deliberate color exception:** The DRAFT_AUCTION timer keeps the current green/yellow/red readability model from `shop-auction-ui.md`. This is a deliberate UI state exception to the art bible's amber/crimson urgency ramp, not a silent normalization. The exception is constrained to the auction timer bar only. Auction border heat still follows the blue to amber to crimson-amber price ramp.

**Bid confirmation decision:** Accessibility requirements call for optional bid confirmation, but the interaction design is unresolved. No confirmation-step production assets are created in this pass.

---

## Assets

| Asset ID | Name | Category | Format / Dimensions | Naming | Status |
|---|---|---|---|---|---|
| ASSET-175 | Auction Ambient Urgency Tone Loop | Audio | OGG Vorbis loop / WAV master | `audio_auction_urgency_loop.ogg` | Needed |
| ASSET-176 | Accepted Bid Ascending SFX | Audio | OGG Vorbis / WAV master | `audio_auction_bid_accepted.ogg` | Needed |
| ASSET-177 | Auction Red-Zone Countdown Tick Cue | Audio / Reuse | Uses ASSET-021 tick file, auction-owned trigger | `audio_countdown_tick_loop.ogg` | Needed / Reuse |
| ASSET-178 | Timer Reset Reverse-Tick SFX | Audio | OGG Vorbis / WAV master | `audio_auction_timer_extend.ogg` | Needed |
| ASSET-179 | Auction Won By Self Sting | Audio | OGG Vorbis / WAV master | `audio_auction_won_self.ogg` | Needed |
| ASSET-180 | Auction Won By Opponent Sting | Audio | OGG Vorbis / WAV master | `audio_auction_won_opponent.ogg` | Needed |
| ASSET-181 | No-Bid Card Gone SFX | Audio | OGG Vorbis / WAV master | `audio_auction_no_bid_card_gone.ogg` | Needed |
| ASSET-182 | Auction Timer Bar Material Exception | UI Material | Green/yellow/red zones, 300ms cross-fade | N/A | Needed |
| ASSET-183 | Local Expiry Awaiting-Settlement Pulse | UI Material / Animation | 0% timer pulse after 500ms | N/A | Needed |
| ASSET-184 | Auction Disconnect Grace Overlay | UI | Bevy UI chip / optional PNG icon | `ui_auction_disconnect_waiting_hud.png` if icon needed | Placeholder |

### Visual Direction

- **ASSET-182**: DRAFT_AUCTION timer fill uses green above 10s, yellow from 5-10s, red below 5s. Fill length and numeric seconds remain the primary accessibility signal.
- **ASSET-183**: frozen 0% timer state displays "Auction ending..." immediately, then subtle timer pulse after 500ms, then "Awaiting server..." after 1500ms. No panic red full-screen treatment.
- **ASSET-184**: overlay sits over bid controls only during disconnect grace. Current leader, price, and timer remain visible. Bid buttons disabled while grace is active.

### Sonic Direction

- **Ambient urgency loop** starts on DRAFT_AUCTION entry and fades out over the AUCTION to SHOP transition.
- **Bid accepted** is short and ascending; rapid bids should form an escalating pitch series without becoming musical clutter.
- **Timer extension** is a brief reverse-tick or inhaling clock gesture.
- **Self win / opponent win / no-bid** must be distinct without becoming victory/defeat fanfare. The auction is a read, not a match result.
