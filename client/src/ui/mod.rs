// UI layer: Presentation — board, hand, shop, HUD (M2+)
pub mod design_tokens;
pub mod hand;
pub mod hud;
pub mod lobby;
pub mod phase_banner;
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

// PROMPT 1404 / `S19-UI-PHASE-CHANGE-BANNER-001` — transient centered
// banner painted on every major `RoundPhase` transition. Re-exported here
// so `PresentationPlugin` registers it with one
// `app.add_plugins(crate::ui::PhaseBannerPlugin)` line alongside the other
// in-session UI plugins.
pub use phase_banner::{
    phase_banner_label_for, PhaseBannerLabel, PhaseBannerPanel, PhaseBannerPlugin,
    PhaseBannerRoot, PHASE_BANNER_BACKGROUND_COLOR, PHASE_BANNER_BORDER_COLOR,
    PHASE_BANNER_LIFETIME, PHASE_BANNER_MAX_WIDTH_PERCENT, PHASE_BANNER_MAX_WIDTH_PX,
    PHASE_BANNER_MIN_HEIGHT_PX, PHASE_BANNER_TEXT_COLOR,
};
