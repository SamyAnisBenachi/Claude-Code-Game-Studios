use bevy::prelude::*;
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::{CardCatalog, CardId, Rarity};
use shared::protocol::{
    C2SPurchaseCard, C2SSignalReady, CardSource, ReliableChannel, RoundPhase, S2CCardAcquired,
    S2CDraftOffering,
};

use crate::presentation::PlayerEconomyView;
use crate::state::{ClientState, CurrentClientPhase};

pub const SHOP_AUCTION_UI_PANEL_ROOT_COUNT: usize = 6;
pub const SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT: usize = 9;
pub const BID_INCREMENTS: [u32; 3] = [1, 3, 5];

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShopAuctionUiSystemSet {
    PhaseTransition,
    MessageDrain,
    Input,
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

#[derive(Resource, Default, Debug, Clone)]
pub struct ShopAuctionCardCatalog {
    pub cards: CardCatalog,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftHandView {
    pub hand_size: usize,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ShopAuctionUiOutboundMessages {
    pub purchase_cards: Vec<C2SPurchaseCard>,
    pub ready_signals: Vec<C2SSignalReady>,
    pub gold_counter_flash_requests: u32,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionDraftInitialState {
    pub offering_loaded: bool,
    pub ready_signalled: bool,
    pending_confirmed_purchases: Vec<CardId>,
}

impl ShopAuctionDraftInitialState {
    fn reset_phase_state(&mut self) {
        self.offering_loaded = false;
        self.ready_signalled = false;
        self.pending_confirmed_purchases.clear();
    }

    fn queue_purchase_confirmation(&mut self, card_id: CardId) {
        self.pending_confirmed_purchases.push(card_id);
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ShopAuctionUiEntities {
    pub root: Entity,
    pub draft_offering_panel: Entity,
    pub draft_initial_slots: [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    pub draft_initial_bought_overlays: [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    pub draft_initial_ready_button: Entity,
    pub draft_initial_ready_status: Entity,
    pub draft_initial_hand_full_banner: Entity,
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotIndex(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotCard(pub CardId);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DraftInitialSlotCardName(pub String);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotGoldCost(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotRarity(pub Rarity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftInitialSlotState {
    Available,
    Pending,
    HandFullLocked,
    Purchased,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialBoughtOverlay {
    pub slot_index: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialReadyButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialReadyStatus;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialHandFullBanner;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionDraftOfferingReceived {
    pub card_ids: Vec<CardId>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionCardAcquiredReceived {
    pub card_id: CardId,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftSlotClicked {
    pub slot: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftReadyButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionGoldCounterFlashRequested;

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
            .init_resource::<ShopAuctionCardCatalog>()
            .init_resource::<ShopAuctionDraftHandView>()
            .init_resource::<ShopAuctionUiOutboundMessages>()
            .init_resource::<ShopAuctionDraftInitialState>()
            .init_resource::<PlayerEconomyView>()
            .add_message::<ShopAuctionDraftOfferingReceived>()
            .add_message::<ShopAuctionCardAcquiredReceived>()
            .add_message::<ShopAuctionDraftSlotClicked>()
            .add_message::<ShopAuctionDraftReadyButtonClicked>()
            .add_message::<ShopAuctionGoldCounterFlashRequested>()
            .configure_sets(
                Update,
                (
                    ShopAuctionUiSystemSet::PhaseTransition,
                    ShopAuctionUiSystemSet::MessageDrain,
                    ShopAuctionUiSystemSet::Input,
                    ShopAuctionUiSystemSet::StateSync,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .add_systems(OnEnter(ClientState::InSession), spawn_shop_auction_ui)
            .add_systems(OnExit(ClientState::InSession), despawn_shop_auction_ui)
            .add_systems(
                Update,
                (
                    shop_auction_ui_phase_transition_system
                        .in_set(ShopAuctionUiSystemSet::PhaseTransition),
                    (
                        drain_draft_offering_receiver_system,
                        drain_card_acquired_receiver_system,
                        handle_draft_offering_system,
                        handle_card_acquired_system,
                        apply_draft_initial_purchase_confirmations_system,
                    )
                        .chain()
                        .in_set(ShopAuctionUiSystemSet::MessageDrain),
                    (
                        handle_draft_initial_slot_click_system,
                        handle_draft_initial_ready_click_system,
                    )
                        .chain()
                        .in_set(ShopAuctionUiSystemSet::Input),
                    sync_draft_initial_panel_system.in_set(ShopAuctionUiSystemSet::StateSync),
                ),
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

pub fn sort_draft_offering_card_ids(card_ids: &[CardId], catalog: &CardCatalog) -> Vec<CardId> {
    let mut indexed_cards = card_ids
        .iter()
        .copied()
        .take(SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT)
        .enumerate()
        .collect::<Vec<_>>();

    indexed_cards.sort_by(|(left_index, left_id), (right_index, right_id)| {
        let left = catalog.get(left_id);
        let right = catalog.get(right_id);
        let left_rank = left.map_or(0, |card| rarity_sort_rank(card.rarity));
        let right_rank = right.map_or(0, |card| rarity_sort_rank(card.rarity));
        let left_cost = left.map_or(0, |card| card.cost);
        let right_cost = right.map_or(0, |card| card.cost);

        right_rank
            .cmp(&left_rank)
            .then_with(|| right_cost.cmp(&left_cost))
            .then_with(|| left_index.cmp(right_index))
    });

    indexed_cards
        .into_iter()
        .map(|(_index, card_id)| card_id)
        .collect()
}

pub fn shop_auction_ui_phase_transition_system(
    current: Res<CurrentClientPhase>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut mode: ResMut<ShopAuctionUiMode>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut visibility: Query<&mut Visibility>,
) {
    if !current.is_changed() {
        return;
    }

    let next_mode = ShopAuctionUiMode::from_phase(current.phase);
    *mode = next_mode;

    if next_mode != ShopAuctionUiMode::DraftOffering {
        draft_state.reset_phase_state();
    }

    let Some(entities) = entities else {
        return;
    };

    set_visibility(
        &mut visibility,
        entities.root,
        visibility_for(
            next_mode != ShopAuctionUiMode::Inactive
                && (next_mode != ShopAuctionUiMode::DraftOffering || draft_state.offering_loaded),
        ),
    );
    set_visibility(
        &mut visibility,
        entities.draft_offering_panel,
        visibility_for(
            next_mode == ShopAuctionUiMode::DraftOffering && draft_state.offering_loaded,
        ),
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

pub fn drain_draft_offering_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CDraftOffering>>,
    mut writer: MessageWriter<ShopAuctionDraftOfferingReceived>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            writer.write(ShopAuctionDraftOfferingReceived {
                card_ids: message.card_ids,
            });
        }
    }
}

pub fn drain_card_acquired_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CCardAcquired>>,
    mut writer: MessageWriter<ShopAuctionCardAcquiredReceived>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            if message.source == CardSource::DraftInitial {
                writer.write(ShopAuctionCardAcquiredReceived {
                    card_id: message.card_id,
                });
            }
        }
    }
}

pub fn handle_draft_offering_system(
    mode: Res<ShopAuctionUiMode>,
    catalog: Res<ShopAuctionCardCatalog>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut offerings: MessageReader<ShopAuctionDraftOfferingReceived>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut commands: Commands,
    mut draft_ui: ParamSet<(
        Query<(&DraftInitialSlotIndex, &mut Text, &mut Visibility)>,
        Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
    )>,
) {
    let Some(entities) = entities else {
        for _offering in offerings.read() {}
        return;
    };

    for offering in offerings.read() {
        draft_state.offering_loaded = true;
        draft_state.ready_signalled = false;
        draft_state.pending_confirmed_purchases.clear();

        let sorted_card_ids = sort_draft_offering_card_ids(&offering.card_ids, &catalog.cards);

        {
            let mut slots = draft_ui.p0();
            for slot_entity in entities.draft_initial_slots {
                let Ok((slot_index, mut text, mut visibility)) = slots.get_mut(slot_entity) else {
                    continue;
                };

                let Some(card_id) = sorted_card_ids.get(slot_index.0 as usize).copied() else {
                    clear_draft_initial_slot(
                        &mut commands,
                        slot_entity,
                        &mut text,
                        &mut visibility,
                    );
                    continue;
                };

                let card = catalog.cards.get(&card_id);
                let card_name = card
                    .map(|card| card.name_en.clone())
                    .unwrap_or_else(|| format!("Card {}", card_id.0));
                let cost = card.map_or(0, |card| card.cost);
                let rarity = card.map_or(Rarity::Common, |card| card.rarity);

                text.0.clear();
                text.0
                    .push_str(&format!("{}\n{}g", card_name.as_str(), cost));
                commands.entity(slot_entity).insert((
                    DraftInitialSlotCard(card_id),
                    DraftInitialSlotCardName(card_name),
                    DraftInitialSlotGoldCost(cost),
                    DraftInitialSlotRarity(rarity),
                    DraftInitialSlotState::Available,
                ));
                *visibility = visibility_for(draft_initial_active(&mode, &draft_state));
            }
        }

        let mut overlays = draft_ui.p1();
        for (overlay, mut visibility) in &mut overlays {
            let is_known_overlay =
                (overlay.slot_index as usize) < SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT;
            if is_known_overlay {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn handle_card_acquired_system(
    mut acquisitions: MessageReader<ShopAuctionCardAcquiredReceived>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
) {
    for acquisition in acquisitions.read() {
        draft_state.queue_purchase_confirmation(acquisition.card_id);
    }
}

pub fn apply_draft_initial_purchase_confirmations_system(
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    mut hand_view: ResMut<ShopAuctionDraftHandView>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut commands: Commands,
    mut slots: Query<(
        Entity,
        &DraftInitialSlotIndex,
        &DraftInitialSlotCard,
        &mut DraftInitialSlotState,
    )>,
    mut overlays: Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
) {
    if !draft_initial_active(&mode, &draft_state) || !economy.initialized {
        return;
    }

    let pending_confirmations = std::mem::take(&mut draft_state.pending_confirmed_purchases);
    if pending_confirmations.is_empty() {
        return;
    }

    let mut unapplied_confirmations = Vec::new();
    for card_id in pending_confirmations {
        let Some((slot_entity, slot_index)) =
            mark_confirmed_purchase(card_id, &mut commands, &mut slots)
        else {
            unapplied_confirmations.push(card_id);
            continue;
        };

        commands
            .entity(slot_entity)
            .remove::<PendingDraftInitialPurchase>();
        set_bought_overlay_visibility(slot_index, Visibility::Visible, &mut overlays);
        hand_view.hand_size = hand_view
            .hand_size
            .saturating_add(1)
            .min(SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT + 1);
    }

    draft_state.pending_confirmed_purchases = unapplied_confirmations;
}

pub fn handle_draft_initial_slot_click_system(
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    hand_view: Res<ShopAuctionDraftHandView>,
    draft_state: Res<ShopAuctionDraftInitialState>,
    mut clicks: MessageReader<ShopAuctionDraftSlotClicked>,
    mut slots: Query<(
        &DraftInitialSlotCard,
        &DraftInitialSlotGoldCost,
        &DraftInitialSlotState,
    )>,
    mut senders: Query<&mut MessageSender<C2SPurchaseCard>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
    mut commands: Commands,
    mut flash_writer: MessageWriter<ShopAuctionGoldCounterFlashRequested>,
) {
    for click in clicks.read() {
        if !draft_initial_active(&mode, &draft_state) {
            continue;
        }

        let Ok((card, cost, slot_state)) = slots.get_mut(click.slot) else {
            continue;
        };

        if *slot_state != DraftInitialSlotState::Available {
            continue;
        }

        if hand_view.hand_size >= 10 {
            commands
                .entity(click.slot)
                .insert(DraftInitialSlotState::HandFullLocked);
            continue;
        }

        if cost.0 > economy.gold {
            outbound.gold_counter_flash_requests =
                outbound.gold_counter_flash_requests.saturating_add(1);
            flash_writer.write(ShopAuctionGoldCounterFlashRequested);
            continue;
        }

        let message = C2SPurchaseCard { card_id: card.0 };
        if let Ok(mut sender) = senders.single_mut() {
            sender.send::<ReliableChannel>(message.clone());
        }
        outbound.purchase_cards.push(message);
        commands
            .entity(click.slot)
            .insert((DraftInitialSlotState::Pending, PendingDraftInitialPurchase));
    }
}

pub fn handle_draft_initial_ready_click_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    mode: Res<ShopAuctionUiMode>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut clicks: MessageReader<ShopAuctionDraftReadyButtonClicked>,
    mut senders: Query<&mut MessageSender<C2SSignalReady>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if click.button != entities.draft_initial_ready_button
            || !draft_initial_active(&mode, &draft_state)
        {
            continue;
        }

        let message = C2SSignalReady {
            retract: draft_state.ready_signalled,
        };
        if let Ok(mut sender) = senders.single_mut() {
            sender.send::<ReliableChannel>(message.clone());
        }
        outbound.ready_signals.push(message);
        draft_state.ready_signalled = !draft_state.ready_signalled;
    }
}

pub fn sync_draft_initial_panel_system(
    mode: Res<ShopAuctionUiMode>,
    hand_view: Res<ShopAuctionDraftHandView>,
    draft_state: Res<ShopAuctionDraftInitialState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility>,
        Query<
            (
                Entity,
                Option<&DraftInitialSlotCard>,
                &mut DraftInitialSlotState,
                &mut Visibility,
            ),
            With<DraftInitialSlotIndex>,
        >,
        Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
    )>,
    mut texts: Query<&mut Text>,
    mut commands: Commands,
) {
    let Some(entities) = entities else {
        return;
    };

    let active = draft_initial_active(&mode, &draft_state);
    {
        let mut visibility = visibility_sets.p0();
        if *mode == ShopAuctionUiMode::DraftOffering {
            set_visibility(&mut visibility, entities.root, visibility_for(active));
        }
        set_visibility(
            &mut visibility,
            entities.draft_offering_panel,
            visibility_for(active),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_ready_button,
            visibility_for(active),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_ready_status,
            visibility_for(active && draft_state.ready_signalled),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_hand_full_banner,
            visibility_for(active && hand_view.hand_size >= 10),
        );
    }

    {
        let mut slots = visibility_sets.p1();
        for (slot_entity, card, mut slot_state, mut visibility) in &mut slots {
            if !active || card.is_none() {
                *visibility = Visibility::Hidden;
                continue;
            }

            *visibility = Visibility::Visible;
            if hand_view.hand_size >= 10 && *slot_state != DraftInitialSlotState::Purchased {
                *slot_state = DraftInitialSlotState::HandFullLocked;
                commands
                    .entity(slot_entity)
                    .remove::<PendingDraftInitialPurchase>();
            }
        }
    }

    if !active {
        let mut overlays = visibility_sets.p2();
        for (_overlay, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
    }

    if let Ok(mut text) = texts.get_mut(entities.draft_initial_ready_button) {
        text.0.clear();
        if draft_state.ready_signalled {
            text.0.push_str("Retract Ready");
        } else {
            text.0.push_str("Ready");
        }
    }

    if let Ok(mut text) = texts.get_mut(entities.draft_initial_ready_status) {
        text.0.clear();
        if draft_state.ready_signalled {
            text.0.push_str("Waiting for opponent...");
        }
    }
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
    let (draft_initial_slots, draft_initial_bought_overlays) =
        spawn_draft_initial_grid(&mut commands, draft_offering_panel);
    let draft_initial_ready_button =
        spawn_draft_initial_ready_button(&mut commands, draft_offering_panel);
    let draft_initial_ready_status =
        spawn_draft_initial_status_text(&mut commands, draft_offering_panel);
    let draft_initial_hand_full_banner =
        spawn_draft_initial_hand_full_banner(&mut commands, draft_offering_panel);
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
        draft_initial_slots,
        draft_initial_bought_overlays,
        draft_initial_ready_button,
        draft_initial_ready_status,
        draft_initial_hand_full_banner,
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct PendingDraftInitialPurchase;

fn spawn_draft_initial_grid(
    commands: &mut Commands,
    parent: Entity,
) -> (
    [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
) {
    let mut overlays = Vec::with_capacity(SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT);
    let slots = std::array::from_fn(|index| {
        let slot = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index}")),
                ShopAuctionUiEntity,
                DraftInitialSlotIndex(index as u8),
                draft_initial_slot_node(index),
                Text::new(""),
                shop_auction_text_font(14.0),
                TextColor(Color::srgb(0.92, 0.94, 0.96)),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .id();

        let overlay = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index} Bought Overlay")),
                ShopAuctionUiEntity,
                DraftInitialBoughtOverlay {
                    slot_index: index as u8,
                },
                Text::new("BOUGHT"),
                shop_auction_text_font(14.0),
                TextColor(Color::srgb(1.0, 0.94, 0.78)),
                overlay_text_node(),
                Visibility::Hidden,
                ChildOf(slot),
            ))
            .id();
        overlays.push(overlay);
        slot
    });

    let overlays = overlays
        .try_into()
        .expect("draft grid should always create exactly 9 overlays");
    (slots, overlays)
}

fn spawn_draft_initial_ready_button(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Draft Ready Button"),
            ShopAuctionUiEntity,
            DraftInitialReadyButton,
            Text::new("Ready"),
            shop_auction_text_font(16.0),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
            draft_initial_ready_button_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_status_text(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Draft Ready Status"),
            ShopAuctionUiEntity,
            DraftInitialReadyStatus,
            Text::new(""),
            shop_auction_text_font(13.0),
            TextColor(Color::srgb(0.80, 0.86, 0.94)),
            draft_initial_status_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_hand_full_banner(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Draft Hand Full Banner"),
            ShopAuctionUiEntity,
            DraftInitialHandFullBanner,
            Text::new("Hand full - cannot buy more cards."),
            shop_auction_text_font(14.0),
            TextColor(Color::srgb(1.0, 0.78, 0.55)),
            draft_initial_hand_full_banner_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
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

fn draft_initial_slot_node(index: usize) -> Node {
    let column = index % 3;
    let row = index / 3;
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(96.0 + column as f32 * 132.0),
        top: Val::Px(30.0 + row as f32 * 66.0),
        width: Val::Px(120.0),
        height: Val::Px(56.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn overlay_text_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(22.0),
        top: Val::Px(18.0),
        ..default()
    }
}

fn draft_initial_ready_button_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(58.0),
        width: Val::Px(132.0),
        height: Val::Px(36.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_status_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(100.0),
        width: Val::Px(180.0),
        height: Val::Px(28.0),
        ..default()
    }
}

fn draft_initial_hand_full_banner_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(138.0),
        width: Val::Px(260.0),
        height: Val::Px(30.0),
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

fn draft_initial_active(mode: &ShopAuctionUiMode, state: &ShopAuctionDraftInitialState) -> bool {
    *mode == ShopAuctionUiMode::DraftOffering && state.offering_loaded
}

fn rarity_sort_rank(rarity: Rarity) -> u8 {
    match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
        Rarity::Legendary => 4,
    }
}

fn clear_draft_initial_slot(
    commands: &mut Commands,
    entity: Entity,
    text: &mut Text,
    visibility: &mut Visibility,
) {
    text.0.clear();
    *visibility = Visibility::Hidden;
    commands.entity(entity).remove::<(
        DraftInitialSlotCard,
        DraftInitialSlotCardName,
        DraftInitialSlotGoldCost,
        DraftInitialSlotRarity,
        DraftInitialSlotState,
        PendingDraftInitialPurchase,
    )>();
}

fn mark_confirmed_purchase(
    card_id: CardId,
    commands: &mut Commands,
    slots: &mut Query<(
        Entity,
        &DraftInitialSlotIndex,
        &DraftInitialSlotCard,
        &mut DraftInitialSlotState,
    )>,
) -> Option<(Entity, u8)> {
    for (entity, slot_index, slot_card, mut slot_state) in slots.iter_mut() {
        if slot_card.0 != card_id || *slot_state == DraftInitialSlotState::Purchased {
            continue;
        }

        *slot_state = DraftInitialSlotState::Purchased;
        commands
            .entity(entity)
            .remove::<PendingDraftInitialPurchase>();
        return Some((entity, slot_index.0));
    }

    None
}

fn set_bought_overlay_visibility(
    slot_index: u8,
    target_visibility: Visibility,
    overlays: &mut Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
) {
    for (overlay, mut visibility) in overlays.iter_mut() {
        if overlay.slot_index == slot_index {
            *visibility = target_visibility;
            return;
        }
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
