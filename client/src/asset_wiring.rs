use bevy::prelude::*;
use shared::card::{CardCatalog, CardData, ClassId, Rarity};

use crate::state::ClientState;

const CARD_DATA_JSON: &str = include_str!("../../assets/data/cards.json");

// ── Shared fallback ───────────────────────────────────────────────────────────
pub const PLACEHOLDER_FALLBACK_ASSET: &str = "art/ui/shared/ui_placeholder_1x1_white.png";

// ── Card UI (hand fan chrome) ─────────────────────────────────────────────────
pub const CARD_FRAME_COMMON_HAND_ASSET: &str = "art/ui/card/ui_card_frame_common_hand.png";
pub const CARD_FRAME_RARE_HAND_ASSET: &str = "art/ui/card/ui_card_frame_rare_hand.png";
pub const CARD_FRAME_EPIC_HAND_ASSET: &str = "art/ui/card/ui_card_frame_epic_hand.png";
pub const CARD_FRAME_LEGENDARY_HAND_ASSET: &str = "art/ui/card/ui_card_frame_legendary_hand.png";

pub const STAT_BADGE_ATK_ASSET: &str = "art/ui/card/ui_stat_badge_atk.png";
pub const STAT_BADGE_HP_ASSET: &str = "art/ui/card/ui_stat_badge_hp.png";
pub const STAT_BADGE_MP_ASSET: &str = "art/ui/card/ui_stat_badge_mp.png";
pub const STAT_BADGE_AR_ASSET: &str = "art/ui/card/ui_stat_badge_ar.png";

pub const RARITY_ICON_COMMON_ASSET: &str = "art/ui/card/ui_rarity_common_icon.png";
pub const RARITY_ICON_RARE_ASSET: &str = "art/ui/card/ui_rarity_rare_icon.png";
pub const RARITY_ICON_EPIC_ASSET: &str = "art/ui/card/ui_rarity_epic_icon.png";
pub const RARITY_ICON_LEGENDARY_ASSET: &str = "art/ui/card/ui_rarity_legendary_icon.png";

pub const CLASS_TYPE_ICON_IOP_ASSET: &str = "art/ui/card/ui_class_iop_type_icon.png";
pub const CLASS_TYPE_ICON_CRA_ASSET: &str = "art/ui/card/ui_class_cra_type_icon.png";
pub const CLASS_TYPE_ICON_SACRIER_ASSET: &str = "art/ui/card/ui_class_sacrier_type_icon.png";
pub const CLASS_TYPE_ICON_XELOR_ASSET: &str = "art/ui/card/ui_class_xelor_type_icon.png";
pub const CLASS_TYPE_ICON_ECAFLIP_ASSET: &str = "art/ui/card/ui_class_ecaflip_type_icon.png";
pub const CLASS_TYPE_ICON_SADIDA_ASSET: &str = "art/ui/card/ui_class_sadida_type_icon.png";
pub const CLASS_TYPE_ICON_NEUTRAL_ASSET: &str = "art/ui/card/ui_class_neutral_type_icon.png";

// ── Shop / Auction UI ─────────────────────────────────────────────────────────
pub const SHOP_PANEL_CHROME_ASSET: &str = "art/ui/shop/ui_shop_panel_chrome.png";
pub const SHOP_SLOT_WELL_IDLE_ASSET: &str = "art/ui/shop/ui_slot_well_idle.png";

pub const BID_BUTTON_NORMAL_ASSET: &str = "art/ui/auction/ui_bid_button_normal.png";
pub const BID_BUTTON_HOVER_ASSET: &str = "art/ui/auction/ui_bid_button_hover.png";
pub const BID_BUTTON_DISABLED_ASSET: &str = "art/ui/auction/ui_bid_button_disabled.png";

// ── HUD ───────────────────────────────────────────────────────────────────────
pub const HUD_FIGURINE_IOP_ASSET: &str = "art/ui/hud/ui_class_figurine_iop.png";
pub const HUD_FIGURINE_CRA_ASSET: &str = "art/ui/hud/ui_class_figurine_cra.png";
pub const HUD_FIGURINE_SACRIER_ASSET: &str = "art/ui/hud/ui_class_figurine_sacrier.png";
pub const HUD_FIGURINE_XELOR_ASSET: &str = "art/ui/hud/ui_class_figurine_xelor.png";
pub const HUD_FIGURINE_ECAFLIP_ASSET: &str = "art/ui/hud/ui_class_figurine_ecaflip.png";
pub const HUD_FIGURINE_SADIDA_ASSET: &str = "art/ui/hud/ui_class_figurine_sadida.png";
pub const HUD_FIGURINE_NEUTRAL_ASSET: &str = "art/ui/hud/ui_class_figurine_neutral.png";

pub const HUD_PHASE_TIMER_BAR_ASSET: &str = "art/ui/hud/ui_phase_timer_bar.png";

pub const HUD_OBJECTIVE_DOT_ALIVE_ASSET: &str = "art/ui/hud/ui_objective_dot_alive.png";
pub const HUD_OBJECTIVE_DOT_DESTROYED_ASSET: &str = "art/ui/hud/ui_objective_dot_destroyed.png";
pub const HUD_OBJECTIVE_DOT_UNKNOWN_ASSET: &str = "art/ui/hud/ui_objective_dot_unknown.png";
pub const HUD_OBJECTIVE_DOT_FAKE_ASSET: &str = "art/ui/hud/ui_objective_dot_fake.png";

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
pub fn insert_placeholder_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
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
pub fn remove_placeholder_assets(mut commands: Commands) {
    commands.remove_resource::<PlaceholderAssets>();
}

/// Plugin that registers the [`PlaceholderAssets`] lifecycle systems.
pub struct AssetWiringPlugin;

impl Plugin for AssetWiringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::InSession), insert_placeholder_assets)
            .add_systems(OnExit(ClientState::InSession), remove_placeholder_assets);
    }
}

// ── Existing card display art resolution (unchanged) ─────────────────────────

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDisplayArtAsset {
    pub path: &'static str,
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

pub fn default_client_card_catalog() -> CardCatalog {
    serde_json::from_str::<Vec<CardData>>(CARD_DATA_JSON)
        .expect("assets/data/cards.json should deserialize for client display catalog")
        .into_iter()
        .map(|card| (card.id, card))
        .collect()
}

pub fn resolve_card_display_art(
    card: Option<&CardData>,
) -> Result<&'static str, CardDisplayArtFallbackReason> {
    let Some(card) = card else {
        return Err(CardDisplayArtFallbackReason::MissingDisplayAsset);
    };
    let art_id = card.art_id.trim();
    if art_id.is_empty() {
        return Err(CardDisplayArtFallbackReason::NoArtId);
    }

    let path = format!("art/cards/display/card_{art_id}_art_display.png");
    Ok(Box::leak(path.into_boxed_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::card::{ClassId, Rarity};

    #[test]
    fn card_frame_asset_maps_all_rarities() {
        assert_eq!(
            card_frame_asset(Rarity::Common),
            CARD_FRAME_COMMON_HAND_ASSET
        );
        assert_eq!(
            card_frame_asset(Rarity::Uncommon),
            CARD_FRAME_COMMON_HAND_ASSET
        );
        assert_eq!(card_frame_asset(Rarity::Rare), CARD_FRAME_RARE_HAND_ASSET);
        assert_eq!(card_frame_asset(Rarity::Epic), CARD_FRAME_EPIC_HAND_ASSET);
        assert_eq!(
            card_frame_asset(Rarity::Legendary),
            CARD_FRAME_LEGENDARY_HAND_ASSET
        );
    }

    #[test]
    fn rarity_icon_asset_maps_all_rarities() {
        assert_eq!(rarity_icon_asset(Rarity::Common), RARITY_ICON_COMMON_ASSET);
        assert_eq!(
            rarity_icon_asset(Rarity::Uncommon),
            RARITY_ICON_COMMON_ASSET
        );
        assert_eq!(rarity_icon_asset(Rarity::Rare), RARITY_ICON_RARE_ASSET);
        assert_eq!(rarity_icon_asset(Rarity::Epic), RARITY_ICON_EPIC_ASSET);
        assert_eq!(
            rarity_icon_asset(Rarity::Legendary),
            RARITY_ICON_LEGENDARY_ASSET
        );
    }

    #[test]
    fn class_type_icon_asset_maps_all_classes() {
        assert_eq!(
            class_type_icon_asset(ClassId::Iop),
            CLASS_TYPE_ICON_IOP_ASSET
        );
        assert_eq!(
            class_type_icon_asset(ClassId::Cra),
            CLASS_TYPE_ICON_CRA_ASSET
        );
        assert_eq!(
            class_type_icon_asset(ClassId::Sacrier),
            CLASS_TYPE_ICON_SACRIER_ASSET
        );
        assert_eq!(
            class_type_icon_asset(ClassId::Xelor),
            CLASS_TYPE_ICON_XELOR_ASSET
        );
        assert_eq!(
            class_type_icon_asset(ClassId::Ecaflip),
            CLASS_TYPE_ICON_ECAFLIP_ASSET
        );
        assert_eq!(
            class_type_icon_asset(ClassId::Sadida),
            CLASS_TYPE_ICON_SADIDA_ASSET
        );
        assert_eq!(
            class_type_icon_asset(ClassId::Neutral),
            CLASS_TYPE_ICON_NEUTRAL_ASSET
        );
    }

    #[test]
    fn hud_figurine_asset_maps_all_classes() {
        assert_eq!(hud_figurine_asset(ClassId::Iop), HUD_FIGURINE_IOP_ASSET);
        assert_eq!(hud_figurine_asset(ClassId::Cra), HUD_FIGURINE_CRA_ASSET);
        assert_eq!(
            hud_figurine_asset(ClassId::Sacrier),
            HUD_FIGURINE_SACRIER_ASSET
        );
        assert_eq!(hud_figurine_asset(ClassId::Xelor), HUD_FIGURINE_XELOR_ASSET);
        assert_eq!(
            hud_figurine_asset(ClassId::Ecaflip),
            HUD_FIGURINE_ECAFLIP_ASSET
        );
        assert_eq!(
            hud_figurine_asset(ClassId::Sadida),
            HUD_FIGURINE_SADIDA_ASSET
        );
        assert_eq!(
            hud_figurine_asset(ClassId::Neutral),
            HUD_FIGURINE_NEUTRAL_ASSET
        );
    }

    #[test]
    fn board_unit_asset_maps_all_classes() {
        assert_eq!(board_unit_asset(ClassId::Iop), BOARD_UNIT_IOP_ASSET);
        assert_eq!(board_unit_asset(ClassId::Cra), BOARD_UNIT_CRA_ASSET);
        assert_eq!(board_unit_asset(ClassId::Sacrier), BOARD_UNIT_SACRIER_ASSET);
        assert_eq!(board_unit_asset(ClassId::Xelor), BOARD_UNIT_XELOR_ASSET);
        assert_eq!(board_unit_asset(ClassId::Ecaflip), BOARD_UNIT_ECAFLIP_ASSET);
        assert_eq!(board_unit_asset(ClassId::Sadida), BOARD_UNIT_SADIDA_ASSET);
        assert_eq!(board_unit_asset(ClassId::Neutral), BOARD_UNIT_NEUTRAL_ASSET);
    }

    #[test]
    fn lobby_portrait_asset_maps_all_classes() {
        assert_eq!(lobby_portrait_asset(ClassId::Iop), LOBBY_PORTRAIT_IOP_ASSET);
        assert_eq!(lobby_portrait_asset(ClassId::Cra), LOBBY_PORTRAIT_CRA_ASSET);
        assert_eq!(
            lobby_portrait_asset(ClassId::Sacrier),
            LOBBY_PORTRAIT_SACRIER_ASSET
        );
        assert_eq!(
            lobby_portrait_asset(ClassId::Xelor),
            LOBBY_PORTRAIT_XELOR_ASSET
        );
        assert_eq!(
            lobby_portrait_asset(ClassId::Ecaflip),
            LOBBY_PORTRAIT_ECAFLIP_ASSET
        );
        assert_eq!(
            lobby_portrait_asset(ClassId::Sadida),
            LOBBY_PORTRAIT_SADIDA_ASSET
        );
        assert_eq!(
            lobby_portrait_asset(ClassId::Neutral),
            LOBBY_PORTRAIT_NEUTRAL_ASSET
        );
    }

    #[test]
    fn hud_objective_dot_asset_maps_all_states() {
        assert_eq!(
            hud_objective_dot_asset(ObjectiveDotState::Alive),
            HUD_OBJECTIVE_DOT_ALIVE_ASSET
        );
        assert_eq!(
            hud_objective_dot_asset(ObjectiveDotState::Destroyed),
            HUD_OBJECTIVE_DOT_DESTROYED_ASSET
        );
        assert_eq!(
            hud_objective_dot_asset(ObjectiveDotState::Unknown),
            HUD_OBJECTIVE_DOT_UNKNOWN_ASSET
        );
        assert_eq!(
            hud_objective_dot_asset(ObjectiveDotState::Fake),
            HUD_OBJECTIVE_DOT_FAKE_ASSET
        );
    }

    #[test]
    fn bid_button_asset_maps_all_states() {
        assert_eq!(
            bid_button_asset(BidButtonChromeState::Normal),
            BID_BUTTON_NORMAL_ASSET
        );
        assert_eq!(
            bid_button_asset(BidButtonChromeState::Hover),
            BID_BUTTON_HOVER_ASSET
        );
        assert_eq!(
            bid_button_asset(BidButtonChromeState::Disabled),
            BID_BUTTON_DISABLED_ASSET
        );
    }
}
