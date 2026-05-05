use bevy::prelude::*;
use shared::protocol::RoundPhase;

use crate::state::{ClientState, CurrentClientPhase};

pub const SHOP_AUCTION_UI_PANEL_ROOT_COUNT: usize = 6;
pub const BID_INCREMENTS: [u32; 3] = [1, 3, 5];

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShopAuctionUiSystemSet {
    PhaseTransition,
    StateSync,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopAuctionUiMode {
    #[default]
    Inactive,
    DraftOffering,
    Auction,
    Shop,
}

impl ShopAuctionUiMode {
    pub fn from_phase(phase: RoundPhase) -> Self {
        match phase {
            RoundPhase::DraftInitial => Self::DraftOffering,
            RoundPhase::DraftAuction => Self::Auction,
            RoundPhase::DraftShop => Self::Shop,
            RoundPhase::Lobby
            | RoundPhase::Placement
            | RoundPhase::Resolution
            | RoundPhase::GameOver
            | RoundPhase::Handshaking => Self::Inactive,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ShopAuctionUiEntities {
    pub root: Entity,
    pub draft_offering_panel: Entity,
    pub shop_panel: Entity,
    pub auction_panel: Entity,
    pub shop_footer: Entity,
    pub toast_root: Entity,
    pub settlement_overlay: Entity,
}

impl ShopAuctionUiEntities {
    pub fn panel_roots(self) -> [Entity; SHOP_AUCTION_UI_PANEL_ROOT_COUNT] {
        [
            self.draft_offering_panel,
            self.shop_panel,
            self.auction_panel,
            self.shop_footer,
            self.toast_root,
            self.settlement_overlay,
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionUiEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionUiRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopAuctionPanelRoot {
    DraftOffering,
    Shop,
    Auction,
    ShopFooter,
    Toast,
    SettlementOverlay,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionBidButton {
    pub increment: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BidButtonLabel {
    pub total_commitment: u32,
    pub increment: u32,
}

impl BidButtonLabel {
    pub fn text(self) -> String {
        format!("{}g (+{})", self.total_commitment, self.increment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionBorderColorTier {
    PaleInkBlue,
    AuctionAmber,
    DeepAmber,
    CrimsonAmber,
}

impl AuctionBorderColorTier {
    pub fn color(self) -> Color {
        match self {
            Self::PaleInkBlue => Color::srgb_u8(0x2A, 0x4D, 0x8A),
            Self::AuctionAmber => Color::srgb_u8(0xE8, 0x7C, 0x1E),
            Self::DeepAmber => Color::srgb_u8(0xC2, 0x63, 0x0E),
            Self::CrimsonAmber => Color::srgb_u8(0x9C, 0x20, 0x00),
        }
    }
}

pub struct ShopAuctionUiPlugin;

impl Plugin for ShopAuctionUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .init_resource::<CurrentClientPhase>()
            .init_resource::<ShopAuctionUiMode>()
            .configure_sets(
                Update,
                (
                    ShopAuctionUiSystemSet::PhaseTransition,
                    ShopAuctionUiSystemSet::StateSync,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .add_systems(OnEnter(ClientState::InSession), spawn_shop_auction_ui)
            .add_systems(OnExit(ClientState::InSession), despawn_shop_auction_ui)
            .add_systems(
                Update,
                shop_auction_ui_phase_transition_system
                    .in_set(ShopAuctionUiSystemSet::PhaseTransition),
            );
    }
}

pub fn local_free_gold(gold: u32, reserved_gold: u32) -> u32 {
    gold.saturating_sub(reserved_gold)
}

pub fn bid_button_labels(current_price: u32) -> [BidButtonLabel; 3] {
    BID_INCREMENTS.map(|increment| BidButtonLabel {
        total_commitment: current_price.saturating_add(increment),
        increment,
    })
}

pub fn bid_button_label_texts(current_price: u32) -> [String; 3] {
    bid_button_labels(current_price).map(BidButtonLabel::text)
}

pub fn auction_border_color_tier(current_price: u32) -> AuctionBorderColorTier {
    match current_price {
        0..=3 => AuctionBorderColorTier::PaleInkBlue,
        4..=6 => AuctionBorderColorTier::AuctionAmber,
        7..=9 => AuctionBorderColorTier::DeepAmber,
        _ => AuctionBorderColorTier::CrimsonAmber,
    }
}

pub fn shop_auction_ui_phase_transition_system(
    current: Res<CurrentClientPhase>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut mode: ResMut<ShopAuctionUiMode>,
    mut visibility: Query<&mut Visibility>,
) {
    if !current.is_changed() {
        return;
    }

    let next_mode = ShopAuctionUiMode::from_phase(current.phase);
    *mode = next_mode;

    let Some(entities) = entities else {
        return;
    };

    set_visibility(
        &mut visibility,
        entities.root,
        visibility_for(next_mode != ShopAuctionUiMode::Inactive),
    );
    set_visibility(
        &mut visibility,
        entities.draft_offering_panel,
        visibility_for(next_mode == ShopAuctionUiMode::DraftOffering),
    );
    set_visibility(
        &mut visibility,
        entities.shop_panel,
        visibility_for(next_mode == ShopAuctionUiMode::Shop),
    );
    set_visibility(
        &mut visibility,
        entities.auction_panel,
        visibility_for(next_mode == ShopAuctionUiMode::Auction),
    );
    set_visibility(
        &mut visibility,
        entities.shop_footer,
        visibility_for(next_mode == ShopAuctionUiMode::Auction),
    );
    set_visibility(&mut visibility, entities.toast_root, Visibility::Hidden);
    set_visibility(
        &mut visibility,
        entities.settlement_overlay,
        Visibility::Hidden,
    );
}

fn spawn_shop_auction_ui(mut commands: Commands, existing: Option<Res<ShopAuctionUiEntities>>) {
    if existing.is_some() {
        return;
    }

    let root = commands
        .spawn((
            Name::new("Shop Auction UI Root"),
            ShopAuctionUiEntity,
            ShopAuctionUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            Visibility::Hidden,
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands
        .entity(root)
        .insert(bevy::picking::Pickable::IGNORE);

    let draft_offering_panel = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::DraftOffering,
        "Shop Auction Draft Offering Root",
        bottom_panel_node(),
    );
    let shop_panel = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::Shop,
        "Shop Auction Shop Root",
        bottom_panel_node(),
    );
    let auction_panel = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::Auction,
        "Shop Auction Auction Root",
        auction_panel_node(),
    );
    let shop_footer = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::ShopFooter,
        "Shop Auction Footer Root",
        footer_node(),
    );
    let toast_root = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::Toast,
        "Shop Auction Toast Root",
        toast_node(),
    );
    let settlement_overlay = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::SettlementOverlay,
        "Shop Auction Settlement Overlay Root",
        overlay_node(),
    );

    commands.insert_resource(ShopAuctionUiEntities {
        root,
        draft_offering_panel,
        shop_panel,
        auction_panel,
        shop_footer,
        toast_root,
        settlement_overlay,
    });
}

fn despawn_shop_auction_ui(mut commands: Commands, entities: Option<Res<ShopAuctionUiEntities>>) {
    let Some(entities) = entities else {
        return;
    };

    commands.entity(entities.root).despawn();
    commands.remove_resource::<ShopAuctionUiEntities>();
}

fn spawn_panel_root(
    commands: &mut Commands,
    parent: Entity,
    marker: ShopAuctionPanelRoot,
    name: &'static str,
    node: Node,
) -> Entity {
    let root = commands
        .spawn((
            Name::new(name),
            ShopAuctionUiEntity,
            marker,
            node,
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    commands.spawn((
        Name::new(format!("{name} Label")),
        ShopAuctionUiEntity,
        Text::new(""),
        shop_auction_text_font(18.0),
        TextColor(Color::srgb(0.92, 0.94, 0.96)),
        panel_label_node(),
        Visibility::Hidden,
        ChildOf(root),
    ));

    root
}

fn bottom_panel_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        bottom: Val::Px(0.0),
        height: Val::Px(260.0),
        ..default()
    }
}

fn auction_panel_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(80.0),
        bottom: Val::Px(140.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn footer_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        bottom: Val::Px(100.0),
        height: Val::Px(96.0),
        ..default()
    }
}

fn toast_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(24.0),
        bottom: Val::Px(220.0),
        width: Val::Px(260.0),
        height: Val::Px(48.0),
        ..default()
    }
}

fn overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        ..default()
    }
}

fn panel_label_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        ..default()
    }
}

fn shop_auction_text_font(font_size: f32) -> TextFont {
    TextFont {
        font_size,
        ..default()
    }
}

fn visibility_for(visible: bool) -> Visibility {
    if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

fn set_visibility(
    visibility: &mut Query<&mut Visibility>,
    entity: Entity,
    target_visibility: Visibility,
) {
    if let Ok(mut current_visibility) = visibility.get_mut(entity) {
        *current_visibility = target_visibility;
    }
}
