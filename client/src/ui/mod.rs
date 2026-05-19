// UI layer: Presentation — board, hand, shop, HUD (M2+)
pub mod design_tokens;
pub mod hand;
pub mod hud;
pub mod lobby;
pub mod photosensitivity_warning;
pub mod settings;
pub mod shared;
pub mod shop_auction;

pub use crate::card_animations as anim;

// Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) — `PlayArea` is the
// canonical flex container for the in-session middle band. Re-exported here
// so `PresentationPlugin` (and any future ui-wiring caller) registers it
// with one `app.add_plugins(crate::ui::PlayAreaPlugin)` line before
// `HandUiPlugin` / `ShopAuctionUiPlugin`. Consumer spawn systems read the
// `PlayAreaRoot` resource and parent into `PlayArea` instead of their
// historical full-viewport parent.
pub use design_tokens::play_area::{PlayArea, PlayAreaPlugin, PlayAreaRoot, PlayAreaSpawnSet};
