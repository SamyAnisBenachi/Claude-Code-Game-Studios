use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use shared::card::{CardCatalog, CardData, ClassId, Rarity};

use crate::state::{ClientSessionIdentity, ClientState};
use crate::ui::design_tokens::card_slot::card_slot_art_image_mode;

const CARD_DATA_JSON: &str = include_str!("../../assets/data/cards.json");

// ── Shared fallback ───────────────────────────────────────────────────────────
// NO ANALOGUE on disk — repointed to universal placeholder.
pub const PLACEHOLDER_FALLBACK_ASSET: &str = "art/characters/ui_unit_placeholder_default_board.png";

// ── Card UI (hand fan chrome) ─────────────────────────────────────────────────
// Disk has class-keyed card frames in art/ui/hand/, not rarity-keyed in art/ui/card/.
// Map each rarity tier to a class frame as a stand-in placeholder.
pub const CARD_FRAME_COMMON_HAND_ASSET: &str = "art/ui/hand/card_frame_neutral_default_display.png";
pub const CARD_FRAME_RARE_HAND_ASSET: &str = "art/ui/hand/card_frame_cra_default_display.png";
pub const CARD_FRAME_EPIC_HAND_ASSET: &str = "art/ui/hand/card_frame_iop_default_display.png";
pub const CARD_FRAME_LEGENDARY_HAND_ASSET: &str =
    "art/ui/hand/card_frame_sacrier_default_display.png";

pub const STAT_BADGE_ATK_ASSET: &str = "art/ui/hand/ui_badge_atk_default_hud.png";
pub const STAT_BADGE_HP_ASSET: &str = "art/ui/hand/ui_badge_hp_default_hud.png";
pub const STAT_BADGE_MP_ASSET: &str = "art/ui/hand/ui_badge_mana_neutral_default_hud.png";
// NO ANALOGUE on disk — repointed to universal placeholder.
pub const STAT_BADGE_AR_ASSET: &str = "art/characters/ui_unit_placeholder_default_board.png";

pub const RARITY_ICON_COMMON_ASSET: &str = "art/ui/hand/ui_icon_field_common_default_hud.png";
pub const RARITY_ICON_RARE_ASSET: &str = "art/ui/hand/ui_icon_field_rare_default_hud.png";
pub const RARITY_ICON_EPIC_ASSET: &str = "art/ui/hand/ui_icon_trap_epic_default_hud.png";
pub const RARITY_ICON_LEGENDARY_ASSET: &str = "art/ui/hand/ui_icon_field_legendary_default_hud.png";

pub const CLASS_TYPE_ICON_IOP_ASSET: &str = "art/ui/hand/ui_badge_mana_iop_default_hud.png";
pub const CLASS_TYPE_ICON_CRA_ASSET: &str = "art/ui/hand/ui_badge_mana_cra_default_hud.png";
pub const CLASS_TYPE_ICON_SACRIER_ASSET: &str = "art/ui/hand/ui_badge_mana_sacrier_default_hud.png";
pub const CLASS_TYPE_ICON_XELOR_ASSET: &str = "art/ui/hand/ui_badge_mana_xelor_default_hud.png";
pub const CLASS_TYPE_ICON_ECAFLIP_ASSET: &str = "art/ui/hand/ui_badge_mana_ecaflip_default_hud.png";
pub const CLASS_TYPE_ICON_SADIDA_ASSET: &str = "art/ui/hand/ui_badge_mana_sadida_default_hud.png";
pub const CLASS_TYPE_ICON_NEUTRAL_ASSET: &str = "art/ui/hand/ui_badge_mana_neutral_default_hud.png";

// ── Shop / Auction UI ─────────────────────────────────────────────────────────
pub const SHOP_PANEL_CHROME_ASSET: &str = "art/ui/shop/ui_shop_panel_chrome.png";
pub const SHOP_SLOT_WELL_IDLE_ASSET: &str = "art/ui/shop/ui_slot_well_idle.png";

pub const BID_BUTTON_NORMAL_ASSET: &str = "art/ui/auction/ui_bid_button_active.png";
// NO ANALOGUE on disk — repointed to universal placeholder.
pub const BID_BUTTON_HOVER_ASSET: &str = "art/characters/ui_unit_placeholder_default_board.png";
pub const BID_BUTTON_DISABLED_ASSET: &str = "art/ui/auction/ui_bid_button_disabled.png";

// ── Auction tier-border chrome (SLICE-B) ──────────────────────────────────────
// Disk: art/ui/shop_auction/ui_auction_border_tier{1-4}_hud.png
// Maps 1-to-1 with AuctionBorderColorTier in shop_auction/mod.rs:
//   tier 1 ↔ PaleInkBlue (current_price 0–3)
//   tier 2 ↔ AuctionAmber (current_price 4–6)
//   tier 3 ↔ DeepAmber    (current_price 7–9)
//   tier 4 ↔ CrimsonAmber (current_price 10+)
pub const AUCTION_TIER_BORDER_1_ASSET: &str =
    "art/ui/shop_auction/ui_auction_border_tier1_hud.png";
pub const AUCTION_TIER_BORDER_2_ASSET: &str =
    "art/ui/shop_auction/ui_auction_border_tier2_hud.png";
pub const AUCTION_TIER_BORDER_3_ASSET: &str =
    "art/ui/shop_auction/ui_auction_border_tier3_hud.png";
pub const AUCTION_TIER_BORDER_4_ASSET: &str =
    "art/ui/shop_auction/ui_auction_border_tier4_hud.png";

// ── Auction rarity gem icons (SLICE-B) ───────────────────────────────────────
// Disk: art/ui/shop_auction/ui_gem_{rarity}_default_{size}.png
// Two sizes: 24px for compact bid-button slots, 32px for featured-card use.
pub const AUCTION_GEM_RARE_24_ASSET: &str = "art/ui/shop_auction/ui_gem_rare_default_24.png";
pub const AUCTION_GEM_RARE_32_ASSET: &str = "art/ui/shop_auction/ui_gem_rare_default_32.png";
pub const AUCTION_GEM_EPIC_24_ASSET: &str = "art/ui/shop_auction/ui_gem_epic_default_24.png";
pub const AUCTION_GEM_EPIC_32_ASSET: &str = "art/ui/shop_auction/ui_gem_epic_default_32.png";
pub const AUCTION_GEM_LEGENDARY_24_ASSET: &str =
    "art/ui/shop_auction/ui_gem_legendary_default_24.png";
pub const AUCTION_GEM_LEGENDARY_32_ASSET: &str =
    "art/ui/shop_auction/ui_gem_legendary_default_32.png";

// ── HUD ───────────────────────────────────────────────────────────────────────
// Disk has no art/ui/hud/ directory. Map figurines to the class-keyed hand frames.
pub const HUD_FIGURINE_IOP_ASSET: &str = "art/ui/hand/card_frame_iop_default_display.png";
pub const HUD_FIGURINE_CRA_ASSET: &str = "art/ui/hand/card_frame_cra_default_display.png";
pub const HUD_FIGURINE_SACRIER_ASSET: &str = "art/ui/hand/card_frame_sacrier_default_display.png";
pub const HUD_FIGURINE_XELOR_ASSET: &str = "art/ui/hand/card_frame_xelor_default_display.png";
pub const HUD_FIGURINE_ECAFLIP_ASSET: &str = "art/ui/hand/card_frame_ecaflip_default_display.png";
pub const HUD_FIGURINE_SADIDA_ASSET: &str = "art/ui/hand/card_frame_sadida_default_display.png";
pub const HUD_FIGURINE_NEUTRAL_ASSET: &str = "art/ui/hand/card_frame_neutral_default_display.png";

// NO ANALOGUE on disk — repointed to universal placeholder.
pub const HUD_PHASE_TIMER_BAR_ASSET: &str = "art/characters/ui_unit_placeholder_default_board.png";

pub const HUD_OBJECTIVE_DOT_ALIVE_ASSET: &str = "art/board/env_objective_real_reveal_board.png";
// NO ANALOGUE on disk — repointed to universal placeholder.
pub const HUD_OBJECTIVE_DOT_DESTROYED_ASSET: &str =
    "art/characters/ui_unit_placeholder_default_board.png";
pub const HUD_OBJECTIVE_DOT_UNKNOWN_ASSET: &str = "art/board/env_objective_unknown_board.png";
pub const HUD_OBJECTIVE_DOT_FAKE_ASSET: &str = "art/board/env_objective_fake_crack_board.png";

// ── Board characters (world-space Sprite — NOT ImageNode) ─────────────────────
pub const BOARD_UNIT_IOP_ASSET: &str = "art/characters/ui_class_iop_unit_board.png";
pub const BOARD_UNIT_CRA_ASSET: &str = "art/characters/ui_class_cra_unit_board.png";
pub const BOARD_UNIT_SACRIER_ASSET: &str = "art/characters/ui_class_sacrier_unit_board.png";
pub const BOARD_UNIT_XELOR_ASSET: &str = "art/characters/ui_class_xelor_unit_board.png";
pub const BOARD_UNIT_ECAFLIP_ASSET: &str = "art/characters/ui_class_ecaflip_unit_board.png";
pub const BOARD_UNIT_SADIDA_ASSET: &str = "art/characters/ui_class_sadida_unit_board.png";
pub const BOARD_UNIT_NEUTRAL_ASSET: &str = "art/characters/ui_class_neutral_unit_board.png";
pub const BOARD_CHROME_ASSET: &str = "art/board/env_board_chrome_default.png";

// ── Lobby ─────────────────────────────────────────────────────────────────────
// PROMPT 1138 — Bind the canonical per-class portrait slot.
// AUDIT-1129-07 reported that the picker still rendered as "generic blue card
// backs with `?` corner glyphs" because PROMPT 1081 sidestepped the canonical
// `art/ui/lobby/ui_class_portrait_*.png` slot (then byte-identical placeholders)
// in favour of the class-distinct `card_frame_<class>` frames. The canonical
// lobby slot has since been re-stamped with class-distinct content (sourced
// from the `card_frame_<class>` frames), so the picker now binds the canonical
// path. When real portrait art lands the swap is a single file-replace at
// `assets/art/ui/lobby/ui_class_portrait_<class>.png` with no constant edit.
// A class-distinct icon overlay (`class_type_icon_<class>`) is composited on
// each picker tile in `client/src/ui/lobby.rs` so class identity reads at a
// glance until real portrait art is authored.
pub const LOBBY_PORTRAIT_IOP_ASSET: &str = "art/ui/lobby/ui_class_portrait_iop.png";
pub const LOBBY_PORTRAIT_CRA_ASSET: &str = "art/ui/lobby/ui_class_portrait_cra.png";
pub const LOBBY_PORTRAIT_SACRIER_ASSET: &str = "art/ui/lobby/ui_class_portrait_sacrier.png";
pub const LOBBY_PORTRAIT_XELOR_ASSET: &str = "art/ui/lobby/ui_class_portrait_xelor.png";
pub const LOBBY_PORTRAIT_ECAFLIP_ASSET: &str = "art/ui/lobby/ui_class_portrait_ecaflip.png";
pub const LOBBY_PORTRAIT_SADIDA_ASSET: &str = "art/ui/lobby/ui_class_portrait_sadida.png";
pub const LOBBY_PORTRAIT_NEUTRAL_ASSET: &str = "art/ui/lobby/ui_class_portrait_neutral.png";

pub const LOBBY_PLAYER_SLOT_PANEL_ASSET: &str = "art/ui/lobby/ui_player_slot_panel.png";
pub const LOBBY_ROOM_CODE_CHIP_ASSET: &str = "art/ui/lobby/ui_room_code_chip.png";

// ── Enum types for selector functions ────────────────────────────────────────

/// State of an objective dot in the HUD scoreboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectiveDotState {
    Alive,
    Destroyed,
    Unknown,
    Fake,
}

/// Visual chrome state of a bid button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BidButtonChromeState {
    Normal,
    Hover,
    Disabled,
}

// ── Selector functions ────────────────────────────────────────────────────────

/// Returns the path constant for a card frame matching the given rarity.
pub fn card_frame_asset(rarity: Rarity) -> &'static str {
    match rarity {
        Rarity::Common | Rarity::Uncommon => CARD_FRAME_COMMON_HAND_ASSET,
        Rarity::Rare => CARD_FRAME_RARE_HAND_ASSET,
        Rarity::Epic => CARD_FRAME_EPIC_HAND_ASSET,
        Rarity::Legendary => CARD_FRAME_LEGENDARY_HAND_ASSET,
    }
}

/// Returns the path constant for a class type icon matching the given class.
pub fn class_type_icon_asset(class_id: ClassId) -> &'static str {
    match class_id {
        ClassId::Iop => CLASS_TYPE_ICON_IOP_ASSET,
        ClassId::Cra => CLASS_TYPE_ICON_CRA_ASSET,
        ClassId::Sacrier => CLASS_TYPE_ICON_SACRIER_ASSET,
        ClassId::Xelor => CLASS_TYPE_ICON_XELOR_ASSET,
        ClassId::Ecaflip => CLASS_TYPE_ICON_ECAFLIP_ASSET,
        ClassId::Sadida => CLASS_TYPE_ICON_SADIDA_ASSET,
        ClassId::Neutral => CLASS_TYPE_ICON_NEUTRAL_ASSET,
    }
}

/// Returns the path constant for a rarity icon matching the given rarity.
pub fn rarity_icon_asset(rarity: Rarity) -> &'static str {
    match rarity {
        Rarity::Common | Rarity::Uncommon => RARITY_ICON_COMMON_ASSET,
        Rarity::Rare => RARITY_ICON_RARE_ASSET,
        Rarity::Epic => RARITY_ICON_EPIC_ASSET,
        Rarity::Legendary => RARITY_ICON_LEGENDARY_ASSET,
    }
}

/// Returns the path constant for a HUD class figurine matching the given class.
pub fn hud_figurine_asset(class_id: ClassId) -> &'static str {
    match class_id {
        ClassId::Iop => HUD_FIGURINE_IOP_ASSET,
        ClassId::Cra => HUD_FIGURINE_CRA_ASSET,
        ClassId::Sacrier => HUD_FIGURINE_SACRIER_ASSET,
        ClassId::Xelor => HUD_FIGURINE_XELOR_ASSET,
        ClassId::Ecaflip => HUD_FIGURINE_ECAFLIP_ASSET,
        ClassId::Sadida => HUD_FIGURINE_SADIDA_ASSET,
        ClassId::Neutral => HUD_FIGURINE_NEUTRAL_ASSET,
    }
}

/// Returns the path constant for a board unit sprite matching the given class.
pub fn board_unit_asset(class_id: ClassId) -> &'static str {
    match class_id {
        ClassId::Iop => BOARD_UNIT_IOP_ASSET,
        ClassId::Cra => BOARD_UNIT_CRA_ASSET,
        ClassId::Sacrier => BOARD_UNIT_SACRIER_ASSET,
        ClassId::Xelor => BOARD_UNIT_XELOR_ASSET,
        ClassId::Ecaflip => BOARD_UNIT_ECAFLIP_ASSET,
        ClassId::Sadida => BOARD_UNIT_SADIDA_ASSET,
        ClassId::Neutral => BOARD_UNIT_NEUTRAL_ASSET,
    }
}

/// Returns the path constant for a lobby portrait matching the given class.
pub fn lobby_portrait_asset(class_id: ClassId) -> &'static str {
    match class_id {
        ClassId::Iop => LOBBY_PORTRAIT_IOP_ASSET,
        ClassId::Cra => LOBBY_PORTRAIT_CRA_ASSET,
        ClassId::Sacrier => LOBBY_PORTRAIT_SACRIER_ASSET,
        ClassId::Xelor => LOBBY_PORTRAIT_XELOR_ASSET,
        ClassId::Ecaflip => LOBBY_PORTRAIT_ECAFLIP_ASSET,
        ClassId::Sadida => LOBBY_PORTRAIT_SADIDA_ASSET,
        ClassId::Neutral => LOBBY_PORTRAIT_NEUTRAL_ASSET,
    }
}

/// Returns the path constant for an objective dot matching the given state.
pub fn hud_objective_dot_asset(state: ObjectiveDotState) -> &'static str {
    match state {
        ObjectiveDotState::Alive => HUD_OBJECTIVE_DOT_ALIVE_ASSET,
        ObjectiveDotState::Destroyed => HUD_OBJECTIVE_DOT_DESTROYED_ASSET,
        ObjectiveDotState::Unknown => HUD_OBJECTIVE_DOT_UNKNOWN_ASSET,
        ObjectiveDotState::Fake => HUD_OBJECTIVE_DOT_FAKE_ASSET,
    }
}

/// Returns the path constant for a bid button chrome matching the given state.
pub fn bid_button_asset(state: BidButtonChromeState) -> &'static str {
    match state {
        BidButtonChromeState::Normal => BID_BUTTON_NORMAL_ASSET,
        BidButtonChromeState::Hover => BID_BUTTON_HOVER_ASSET,
        BidButtonChromeState::Disabled => BID_BUTTON_DISABLED_ASSET,
    }
}

/// Returns the path constant for an auction tier-border overlay.
///
/// `tier` is 1-indexed (1 = cheapest / PaleInkBlue, 4 = most expensive /
/// CrimsonAmber). Out-of-range values clamp to tier 1.
pub fn auction_tier_border_asset(tier: u8) -> &'static str {
    match tier {
        1 => AUCTION_TIER_BORDER_1_ASSET,
        2 => AUCTION_TIER_BORDER_2_ASSET,
        3 => AUCTION_TIER_BORDER_3_ASSET,
        4 => AUCTION_TIER_BORDER_4_ASSET,
        _ => AUCTION_TIER_BORDER_1_ASSET,
    }
}

// ── PlaceholderAssets resource ────────────────────────────────────────────────

/// Session-scoped resource holding pre-loaded `Handle<Image>` for every
/// presentation placeholder path constant. Inserted on
/// `OnEnter(ClientState::InSession)` and removed on `OnExit`.
///
/// Systems reading this resource must run `in_state(ClientState::InSession)`.
#[derive(Resource)]
pub struct PlaceholderAssets {
    // shared fallback
    pub fallback: Handle<Image>,
    // card UI
    pub card_frame_common: Handle<Image>,
    pub card_frame_rare: Handle<Image>,
    pub card_frame_epic: Handle<Image>,
    pub card_frame_legendary: Handle<Image>,
    pub stat_badge_atk: Handle<Image>,
    pub stat_badge_hp: Handle<Image>,
    pub stat_badge_mp: Handle<Image>,
    pub stat_badge_ar: Handle<Image>,
    pub rarity_icon_common: Handle<Image>,
    pub rarity_icon_rare: Handle<Image>,
    pub rarity_icon_epic: Handle<Image>,
    pub rarity_icon_legendary: Handle<Image>,
    pub class_type_icon_iop: Handle<Image>,
    pub class_type_icon_cra: Handle<Image>,
    pub class_type_icon_sacrier: Handle<Image>,
    pub class_type_icon_xelor: Handle<Image>,
    pub class_type_icon_ecaflip: Handle<Image>,
    pub class_type_icon_sadida: Handle<Image>,
    pub class_type_icon_neutral: Handle<Image>,
    // shop / auction UI
    pub shop_panel_chrome: Handle<Image>,
    pub shop_slot_well_idle: Handle<Image>,
    pub bid_button_normal: Handle<Image>,
    pub bid_button_hover: Handle<Image>,
    pub bid_button_disabled: Handle<Image>,
    // auction tier-border chrome (SLICE-B)
    pub auction_tier_border_1: Handle<Image>,
    pub auction_tier_border_2: Handle<Image>,
    pub auction_tier_border_3: Handle<Image>,
    pub auction_tier_border_4: Handle<Image>,
    // auction rarity gem icons (SLICE-B)
    pub auction_gem_rare_24: Handle<Image>,
    pub auction_gem_rare_32: Handle<Image>,
    pub auction_gem_epic_24: Handle<Image>,
    pub auction_gem_epic_32: Handle<Image>,
    pub auction_gem_legendary_24: Handle<Image>,
    pub auction_gem_legendary_32: Handle<Image>,
    // HUD
    pub hud_figurine_iop: Handle<Image>,
    pub hud_figurine_cra: Handle<Image>,
    pub hud_figurine_sacrier: Handle<Image>,
    pub hud_figurine_xelor: Handle<Image>,
    pub hud_figurine_ecaflip: Handle<Image>,
    pub hud_figurine_sadida: Handle<Image>,
    pub hud_figurine_neutral: Handle<Image>,
    pub hud_phase_timer_bar: Handle<Image>,
    pub hud_objective_dot_alive: Handle<Image>,
    pub hud_objective_dot_destroyed: Handle<Image>,
    pub hud_objective_dot_unknown: Handle<Image>,
    pub hud_objective_dot_fake: Handle<Image>,
    // board characters
    pub board_unit_iop: Handle<Image>,
    pub board_unit_cra: Handle<Image>,
    pub board_unit_sacrier: Handle<Image>,
    pub board_unit_xelor: Handle<Image>,
    pub board_unit_ecaflip: Handle<Image>,
    pub board_unit_sadida: Handle<Image>,
    pub board_unit_neutral: Handle<Image>,
    pub board_chrome: Handle<Image>,
    // lobby
    pub lobby_portrait_iop: Handle<Image>,
    pub lobby_portrait_cra: Handle<Image>,
    pub lobby_portrait_sacrier: Handle<Image>,
    pub lobby_portrait_xelor: Handle<Image>,
    pub lobby_portrait_ecaflip: Handle<Image>,
    pub lobby_portrait_sadida: Handle<Image>,
    pub lobby_portrait_neutral: Handle<Image>,
    pub lobby_player_slot_panel: Handle<Image>,
    pub lobby_room_code_chip: Handle<Image>,
}

/// Inserts [`PlaceholderAssets`] into the world by loading every path constant
/// via [`AssetServer`]. Registered on `OnEnter(ClientState::InSession)`.
pub fn insert_placeholder_assets(
    asset_server: Res<AssetServer>,
    identity: Option<Res<ClientSessionIdentity>>,
    mut commands: Commands,
) {
    tracing::info!(
        state = "InSession",
        player_id = ?identity.as_deref().and_then(|i| i.player_id),
        "client_state_on_enter_in_session",
    );
    commands.insert_resource(PlaceholderAssets {
        fallback: asset_server.load(PLACEHOLDER_FALLBACK_ASSET),
        card_frame_common: asset_server.load(CARD_FRAME_COMMON_HAND_ASSET),
        card_frame_rare: asset_server.load(CARD_FRAME_RARE_HAND_ASSET),
        card_frame_epic: asset_server.load(CARD_FRAME_EPIC_HAND_ASSET),
        card_frame_legendary: asset_server.load(CARD_FRAME_LEGENDARY_HAND_ASSET),
        stat_badge_atk: asset_server.load(STAT_BADGE_ATK_ASSET),
        stat_badge_hp: asset_server.load(STAT_BADGE_HP_ASSET),
        stat_badge_mp: asset_server.load(STAT_BADGE_MP_ASSET),
        stat_badge_ar: asset_server.load(STAT_BADGE_AR_ASSET),
        rarity_icon_common: asset_server.load(RARITY_ICON_COMMON_ASSET),
        rarity_icon_rare: asset_server.load(RARITY_ICON_RARE_ASSET),
        rarity_icon_epic: asset_server.load(RARITY_ICON_EPIC_ASSET),
        rarity_icon_legendary: asset_server.load(RARITY_ICON_LEGENDARY_ASSET),
        class_type_icon_iop: asset_server.load(CLASS_TYPE_ICON_IOP_ASSET),
        class_type_icon_cra: asset_server.load(CLASS_TYPE_ICON_CRA_ASSET),
        class_type_icon_sacrier: asset_server.load(CLASS_TYPE_ICON_SACRIER_ASSET),
        class_type_icon_xelor: asset_server.load(CLASS_TYPE_ICON_XELOR_ASSET),
        class_type_icon_ecaflip: asset_server.load(CLASS_TYPE_ICON_ECAFLIP_ASSET),
        class_type_icon_sadida: asset_server.load(CLASS_TYPE_ICON_SADIDA_ASSET),
        class_type_icon_neutral: asset_server.load(CLASS_TYPE_ICON_NEUTRAL_ASSET),
        shop_panel_chrome: asset_server.load(SHOP_PANEL_CHROME_ASSET),
        shop_slot_well_idle: asset_server.load(SHOP_SLOT_WELL_IDLE_ASSET),
        bid_button_normal: asset_server.load(BID_BUTTON_NORMAL_ASSET),
        bid_button_hover: asset_server.load(BID_BUTTON_HOVER_ASSET),
        bid_button_disabled: asset_server.load(BID_BUTTON_DISABLED_ASSET),
        auction_tier_border_1: asset_server.load(AUCTION_TIER_BORDER_1_ASSET),
        auction_tier_border_2: asset_server.load(AUCTION_TIER_BORDER_2_ASSET),
        auction_tier_border_3: asset_server.load(AUCTION_TIER_BORDER_3_ASSET),
        auction_tier_border_4: asset_server.load(AUCTION_TIER_BORDER_4_ASSET),
        auction_gem_rare_24: asset_server.load(AUCTION_GEM_RARE_24_ASSET),
        auction_gem_rare_32: asset_server.load(AUCTION_GEM_RARE_32_ASSET),
        auction_gem_epic_24: asset_server.load(AUCTION_GEM_EPIC_24_ASSET),
        auction_gem_epic_32: asset_server.load(AUCTION_GEM_EPIC_32_ASSET),
        auction_gem_legendary_24: asset_server.load(AUCTION_GEM_LEGENDARY_24_ASSET),
        auction_gem_legendary_32: asset_server.load(AUCTION_GEM_LEGENDARY_32_ASSET),
        hud_figurine_iop: asset_server.load(HUD_FIGURINE_IOP_ASSET),
        hud_figurine_cra: asset_server.load(HUD_FIGURINE_CRA_ASSET),
        hud_figurine_sacrier: asset_server.load(HUD_FIGURINE_SACRIER_ASSET),
        hud_figurine_xelor: asset_server.load(HUD_FIGURINE_XELOR_ASSET),
        hud_figurine_ecaflip: asset_server.load(HUD_FIGURINE_ECAFLIP_ASSET),
        hud_figurine_sadida: asset_server.load(HUD_FIGURINE_SADIDA_ASSET),
        hud_figurine_neutral: asset_server.load(HUD_FIGURINE_NEUTRAL_ASSET),
        hud_phase_timer_bar: asset_server.load(HUD_PHASE_TIMER_BAR_ASSET),
        hud_objective_dot_alive: asset_server.load(HUD_OBJECTIVE_DOT_ALIVE_ASSET),
        hud_objective_dot_destroyed: asset_server.load(HUD_OBJECTIVE_DOT_DESTROYED_ASSET),
        hud_objective_dot_unknown: asset_server.load(HUD_OBJECTIVE_DOT_UNKNOWN_ASSET),
        hud_objective_dot_fake: asset_server.load(HUD_OBJECTIVE_DOT_FAKE_ASSET),
        board_unit_iop: asset_server.load(BOARD_UNIT_IOP_ASSET),
        board_unit_cra: asset_server.load(BOARD_UNIT_CRA_ASSET),
        board_unit_sacrier: asset_server.load(BOARD_UNIT_SACRIER_ASSET),
        board_unit_xelor: asset_server.load(BOARD_UNIT_XELOR_ASSET),
        board_unit_ecaflip: asset_server.load(BOARD_UNIT_ECAFLIP_ASSET),
        board_unit_sadida: asset_server.load(BOARD_UNIT_SADIDA_ASSET),
        board_unit_neutral: asset_server.load(BOARD_UNIT_NEUTRAL_ASSET),
        board_chrome: asset_server.load(BOARD_CHROME_ASSET),
        lobby_portrait_iop: asset_server.load(LOBBY_PORTRAIT_IOP_ASSET),
        lobby_portrait_cra: asset_server.load(LOBBY_PORTRAIT_CRA_ASSET),
        lobby_portrait_sacrier: asset_server.load(LOBBY_PORTRAIT_SACRIER_ASSET),
        lobby_portrait_xelor: asset_server.load(LOBBY_PORTRAIT_XELOR_ASSET),
        lobby_portrait_ecaflip: asset_server.load(LOBBY_PORTRAIT_ECAFLIP_ASSET),
        lobby_portrait_sadida: asset_server.load(LOBBY_PORTRAIT_SADIDA_ASSET),
        lobby_portrait_neutral: asset_server.load(LOBBY_PORTRAIT_NEUTRAL_ASSET),
        lobby_player_slot_panel: asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET),
        lobby_room_code_chip: asset_server.load(LOBBY_ROOM_CODE_CHIP_ASSET),
    });
}

/// Removes [`PlaceholderAssets`] from the world on session exit.
/// Registered on `OnExit(ClientState::InSession)`.
pub fn remove_placeholder_assets(
    identity: Option<Res<ClientSessionIdentity>>,
    mut commands: Commands,
) {
    tracing::info!(
        state = "Lobby",
        player_id = ?identity.as_deref().and_then(|i| i.player_id),
        "client_state_on_exit_in_session",
    );
    commands.remove_resource::<PlaceholderAssets>();
}

/// Test-only constructor returning a [`PlaceholderAssets`] with every field
/// set to [`Handle::default`]. For `World`-based ECS tests where no
/// [`AssetServer`] is running and the real loader cannot supply handles.
pub fn placeholder_assets_for_tests() -> PlaceholderAssets {
    PlaceholderAssets {
        fallback: Handle::default(),
        card_frame_common: Handle::default(),
        card_frame_rare: Handle::default(),
        card_frame_epic: Handle::default(),
        card_frame_legendary: Handle::default(),
        stat_badge_atk: Handle::default(),
        stat_badge_hp: Handle::default(),
        stat_badge_mp: Handle::default(),
        stat_badge_ar: Handle::default(),
        rarity_icon_common: Handle::default(),
        rarity_icon_rare: Handle::default(),
        rarity_icon_epic: Handle::default(),
        rarity_icon_legendary: Handle::default(),
        class_type_icon_iop: Handle::default(),
        class_type_icon_cra: Handle::default(),
        class_type_icon_sacrier: Handle::default(),
        class_type_icon_xelor: Handle::default(),
        class_type_icon_ecaflip: Handle::default(),
        class_type_icon_sadida: Handle::default(),
        class_type_icon_neutral: Handle::default(),
        shop_panel_chrome: Handle::default(),
        shop_slot_well_idle: Handle::default(),
        bid_button_normal: Handle::default(),
        bid_button_hover: Handle::default(),
        bid_button_disabled: Handle::default(),
        auction_tier_border_1: Handle::default(),
        auction_tier_border_2: Handle::default(),
        auction_tier_border_3: Handle::default(),
        auction_tier_border_4: Handle::default(),
        auction_gem_rare_24: Handle::default(),
        auction_gem_rare_32: Handle::default(),
        auction_gem_epic_24: Handle::default(),
        auction_gem_epic_32: Handle::default(),
        auction_gem_legendary_24: Handle::default(),
        auction_gem_legendary_32: Handle::default(),
        hud_figurine_iop: Handle::default(),
        hud_figurine_cra: Handle::default(),
        hud_figurine_sacrier: Handle::default(),
        hud_figurine_xelor: Handle::default(),
        hud_figurine_ecaflip: Handle::default(),
        hud_figurine_sadida: Handle::default(),
        hud_figurine_neutral: Handle::default(),
        hud_phase_timer_bar: Handle::default(),
        hud_objective_dot_alive: Handle::default(),
        hud_objective_dot_destroyed: Handle::default(),
        hud_objective_dot_unknown: Handle::default(),
        hud_objective_dot_fake: Handle::default(),
        board_unit_iop: Handle::default(),
        board_unit_cra: Handle::default(),
        board_unit_sacrier: Handle::default(),
        board_unit_xelor: Handle::default(),
        board_unit_ecaflip: Handle::default(),
        board_unit_sadida: Handle::default(),
        board_unit_neutral: Handle::default(),
        board_chrome: Handle::default(),
        lobby_portrait_iop: Handle::default(),
        lobby_portrait_cra: Handle::default(),
        lobby_portrait_sacrier: Handle::default(),
        lobby_portrait_xelor: Handle::default(),
        lobby_portrait_ecaflip: Handle::default(),
        lobby_portrait_sadida: Handle::default(),
        lobby_portrait_neutral: Handle::default(),
        lobby_player_slot_panel: Handle::default(),
        lobby_room_code_chip: Handle::default(),
    }
}

/// Test-only helper that drives the
/// `OnEnter(ClientState::InSession)` entry sequence end-to-end inside a
/// `MinimalPlugins`-based fixture so that `spawn_hand_ui` (and any sibling
/// `OnEnter(InSession)` system that depends on [`PlaceholderAssets`]) actually
/// runs and flushes its deferred entity spawns into the world.
///
/// Behavior:
/// 1. Inserts [`placeholder_assets_for_tests`] into the world if absent so
///    `spawn_hand_ui`'s `Option<Res<PlaceholderAssets>>::None` early-return
///    does not silently skip the spawn (see `client::ui::hand::spawn_hand_ui`).
/// 2. Sets `NextState::<ClientState>::Pending(ClientState::InSession)`.
/// 3. Pumps `app.update()` twice: the first cycle applies the state
///    transition + runs `OnEnter(InSession)` systems (which queue spawn
///    commands); the second cycle flushes those deferred commands so
///    downstream queries (e.g., `FanSlotIndex`, `ChildOf<HandCardFrame>`)
///    resolve in the same tick as the assertions.
///
/// **Pre-conditions** the caller's `App` must satisfy:
/// - `MinimalPlugins` is added.
/// - `StatesPlugin` is added and `init_state::<ClientState>()` has run.
/// - The plugin (or hand-picked subset of systems) that registers
///   `spawn_hand_ui` on `OnEnter(ClientState::InSession)` is added — typically
///   [`crate::ui::hand::HandUiPlugin`].
///
/// **Side effects**: this is the only test-only helper that flips `ClientState`
/// to `InSession`; it does NOT also set a `RoundPhase`. Callers that need the
/// hand UI in `Placement` (or any other round phase) must set
/// `CurrentClientPhase`/`ClientPhaseView` themselves and then call
/// `app.update()` once more so the phase-transition systems observe the new
/// phase.
///
/// Mirrors the [`placeholder_assets_for_tests`] precedent (this helper lives
/// next to it deliberately so the cluster of `MinimalPlugins`-fixture
/// prerequisites stays in one place).
///
/// See `docs/architecture/test-fixture-patterns.md` for the canonical pattern
/// and Sprint 11 story `S11-TD-FIXTURE-HAND-UI-ONENTER-001`.
pub fn enter_in_session_via_fixture(app: &mut App) {
    if !app.world().contains_resource::<PlaceholderAssets>() {
        app.insert_resource(placeholder_assets_for_tests());
    }
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app.update();
}

/// Plugin that registers the [`PlaceholderAssets`] lifecycle systems.
pub struct AssetWiringPlugin;

impl Plugin for AssetWiringPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("AssetWiringPlugin loaded");
        app.init_resource::<MissingCardArtWarnings>()
            .add_systems(OnEnter(ClientState::InSession), insert_placeholder_assets)
            .add_systems(
                OnEnter(ClientState::InSession),
                probe_card_display_art_paths.after(insert_placeholder_assets),
            )
            .add_systems(OnExit(ClientState::InSession), remove_placeholder_assets);
    }
}

// ── Card display art resolution + helpers (Sprint 17 S17-UI-CARD-DISPLAY-ART-HELPER-001)

/// Documented sentinel `art_id` whose resolution intentionally routes through
/// the placeholder path rather than the per-card display PNG. Used by test
/// fixtures and by content stubs whose final art is not yet authored
/// (`PAW-TD-*-a` placeholder-art accept-risk). [`resolve_card_display_art`]
/// returns `Ok(CARD_ART_PLACEHOLDER_ASSET.into())` for this sentinel — no
/// `warn!` fires for it; the slot draws the universal placeholder.
pub const CARD_ART_MISSING_SENTINEL: &str = "missing";

/// Placeholder path that resolves to a real 1×1 PNG on disk. Used by
/// [`resolve_card_display_art`] when [`CARD_ART_MISSING_SENTINEL`] is the
/// requested `art_id`. AC4 — missing-art fallthrough path.
pub const CARD_ART_PLACEHOLDER_ASSET: &str = PLACEHOLDER_FALLBACK_ASSET;

/// Marker component holding the resolved display-art path bound to a
/// presentation slot (shop / draft / auction / hand fan / drag sprite).
/// Stores `String` rather than `&'static str` to satisfy
/// SOURCE-1077-03 — the previous `'static` constraint forced
/// [`resolve_card_display_art`] to leak the formatted path on every call.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct CardDisplayArtAsset {
    pub path: String,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDisplayArtFallback {
    pub reason: CardDisplayArtFallbackReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardDisplayArtFallbackReason {
    NoArtId,
    MissingDisplayAsset,
}

/// Session-scoped counter for missing-card-art `warn!` events emitted by
/// [`probe_card_display_art_paths`]. Tests observe this resource rather
/// than capturing the logger directly (AC10). Reset on session exit when
/// the resource is removed and re-initialised on the next session entry.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MissingCardArtWarnings {
    pub count: u32,
}

pub fn default_client_card_catalog() -> CardCatalog {
    serde_json::from_str::<Vec<CardData>>(CARD_DATA_JSON)
        .expect("assets/data/cards.json should deserialize for client display catalog")
        .into_iter()
        .map(|card| (card.id, card))
        .collect()
}

/// Resolves the display-art path for a card.
///
/// Returns `Ok(String)` for resolvable card data (including the documented
/// [`CARD_ART_MISSING_SENTINEL`] which routes through [`CARD_ART_PLACEHOLDER_ASSET`])
/// or `Err(CardDisplayArtFallbackReason)` when the card is absent or its
/// `art_id` is empty / whitespace-only.
///
/// SOURCE-1077-03 — the previous implementation leaked a `Box::leak`-promoted
/// `&'static str` on every call so that `apply_card_display_art` could insert
/// the path into a `'static`-constrained component. The component now stores
/// `String`, removing the leak entirely.
///
/// SOURCE-1077-04 — the existence check that prevents silent blank slots is
/// implemented by [`probe_card_display_art_paths`], which runs on session
/// entry and warns for any `art_id` whose constructed path does not resolve
/// to a real asset. The resolver itself does not block on filesystem I/O.
pub fn resolve_card_display_art(
    card: Option<&CardData>,
) -> Result<String, CardDisplayArtFallbackReason> {
    let Some(card) = card else {
        return Err(CardDisplayArtFallbackReason::MissingDisplayAsset);
    };
    let art_id = card.art_id.trim();
    if art_id.is_empty() {
        return Err(CardDisplayArtFallbackReason::NoArtId);
    }
    if art_id == CARD_ART_MISSING_SENTINEL {
        return Ok(CARD_ART_PLACEHOLDER_ASSET.to_string());
    }

    Ok(format!("art/cards/display/card_{art_id}_art_display.png"))
}

/// Binds a display-art `ImageNode` and marker component to the slot entity
/// presenting `card` (shop / draft / auction / hand fan / drag sprite).
///
/// Sprint 17 S17-UI-CARD-DISPLAY-ART-HELPER-001 single-owner lift site —
/// previously duplicated verbatim under `client/src/ui/shop_auction/mod.rs`
/// and `client/src/ui/hand/mod.rs` (SOURCE-1077-02). Future fixes land here.
///
/// **Chrome preservation strategy (SOURCE-1077-01, AC2)**: on the `Err`
/// branch the helper inserts only [`CardDisplayArtFallback`] and does **not**
/// remove the slot's `ImageNode`. The previous implementation removed
/// `ImageNode` alongside [`CardDisplayArtAsset`], which stripped the spawn-
/// time chrome `ImageNode` (e.g. the shop slot's `SHOP_SLOT_WELL_IDLE_ASSET`
/// well) and left an empty rectangle whenever a card landed whose `art_id`
/// was empty, missing, or unresolvable. [`clear_card_display_art`] applies
/// the same rule on slot vacate.
pub fn apply_card_display_art(
    commands: &mut Commands,
    entity: Entity,
    card: Option<&CardData>,
    asset_server: Option<&AssetServer>,
) {
    match resolve_card_display_art(card) {
        Ok(path) => {
            let mut entity_commands = commands.entity(entity);
            if let Some(asset_server) = asset_server {
                let handle: Handle<Image> = asset_server.load(path.clone());
                // PROMPT 1403 / V-P0-01 / RC-6 — bind the canonical card-art
                // image-mode policy at the single chokepoint. Bevy 0.18's
                // `NodeImageMode` enum has no `Fit` variant (`Auto` /
                // `Stretch` / `Sliced` / `Tiled`); `Auto` is the justified
                // mapping for the story-022 AC2 "Fit or Auto with
                // justification" clause and is sourced from the canonical
                // accessor so future migrations only edit one place.
                entity_commands.insert(ImageNode {
                    image: handle,
                    image_mode: card_slot_art_image_mode(),
                    ..default()
                });
                debug_assert!(
                    !matches!(card_slot_art_image_mode(), NodeImageMode::Stretch),
                    "canonical card-art image_mode must not be Stretch (UI-1129-05 banner-stretch)",
                );
            }
            entity_commands.insert(CardDisplayArtAsset { path });
            entity_commands.remove::<CardDisplayArtFallback>();
        }
        Err(reason) => {
            // Chrome preservation: leave the slot's existing `ImageNode`
            // attached so the spawn-time chrome (e.g. shop well) — or the
            // most recently bound card art — keeps drawing. We only flip
            // the marker components so consumers can observe the fallback.
            commands
                .entity(entity)
                .insert(CardDisplayArtFallback { reason })
                .remove::<CardDisplayArtAsset>();
        }
    }
}

/// Releases the display-art binding on a slot (slot vacate / hand-card
/// discard / shop-slot refresh). Removes only the
/// [`CardDisplayArtAsset`] / [`CardDisplayArtFallback`] markers — never the
/// slot's `ImageNode`, so the spawn-time chrome survives (AC6).
pub fn clear_card_display_art(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<(CardDisplayArtAsset, CardDisplayArtFallback)>();
}

/// Startup probe registered on `OnEnter(ClientState::InSession)` that walks
/// the baked client `CardCatalog`, attempts to load each card's display-art
/// path via [`AssetServer::load`], and warns when the constructed path does
/// not resolve to a real file on the asset reader.
///
/// SOURCE-1077-04 — Bevy's [`AssetServer::load`] returns a `Handle<Image>`
/// even when the underlying file is missing, so the silent blank-slot
/// symptom is only ever surfaced as an `error!` from the async loader and
/// never in association with the card's `art_id`. This probe emits a
/// `warn!` with both `art_id` and `path` so missing-art defects are visible
/// at startup rather than as a silent runtime blank.
///
/// Notes:
/// - The documented [`CARD_ART_MISSING_SENTINEL`] does **not** trigger a
///   warning — its resolution routes through [`CARD_ART_PLACEHOLDER_ASSET`]
///   on purpose (AC7).
/// - On native builds the probe verifies the file via `std::fs::metadata`
///   in addition to scheduling the load; on `wasm32` the filesystem check
///   is skipped (the bundled asset reader makes synchronous probing
///   impossible). The `Handle<Image>` is dropped at end of scope; the
///   probe does not pin the texture into memory.
/// - Warning count is recorded in [`MissingCardArtWarnings`] so tests can
///   observe missing-art events without a logger capture hook (AC10).
pub fn probe_card_display_art_paths(
    asset_server: Option<Res<AssetServer>>,
    mut warnings: ResMut<MissingCardArtWarnings>,
) {
    warnings.count = 0;
    let catalog = default_client_card_catalog();
    for card in catalog.values() {
        let path = match resolve_card_display_art(Some(card)) {
            Ok(path) => path,
            Err(reason) => {
                tracing::warn!(
                    target: "client::asset_wiring",
                    card_id = ?card.id,
                    art_id = %card.art_id,
                    reason = ?reason,
                    "card_display_art_probe_unresolved",
                );
                warnings.count = warnings.count.saturating_add(1);
                continue;
            }
        };

        let art_id_trimmed = card.art_id.trim();
        if art_id_trimmed == CARD_ART_MISSING_SENTINEL {
            // Documented sentinel — routes through placeholder by design.
            continue;
        }

        // Start the async load so the texture is acquired before its first
        // bind. The handle is intentionally discarded — `apply_card_display_art`
        // re-issues a load on demand. We rely on the cache hit for the warm
        // path and accept a cheap re-load on the cold path.
        if let Some(ref asset_server) = asset_server {
            let _handle: Handle<Image> = asset_server.load(path.clone());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let asset_file = std::path::Path::new("assets").join(path.as_str());
            if std::fs::metadata(&asset_file).map_or(true, |meta| !meta.is_file()) {
                tracing::warn!(
                    target: "client::asset_wiring",
                    card_id = ?card.id,
                    art_id = %card.art_id,
                    path = %path,
                    "card_display_art_probe_missing",
                );
                warnings.count = warnings.count.saturating_add(1);
            }
        }
    }
}
