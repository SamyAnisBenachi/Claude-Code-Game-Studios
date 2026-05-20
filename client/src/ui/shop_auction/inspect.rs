//! Shop / auction card inspect overlay consumer.
//!
//! PROMPT 1530 — wires the shared `card_inspect` primitive (PROMPT 1482,
//! adopted by hand / DRAFT_INITIAL in PROMPT 1520) into the shop/auction
//! card surfaces:
//!
//! * [`ShopSlotCard`]            — three shop offer slots.
//! * [`DraftInitialSlotCard`]    — DRAFT_INITIAL 3×3 keep-9 grid (a
//!   shop_auction-owned variant; the hand-side `GridSlotCard` is a
//!   parallel marker covered by [`crate::ui::hand::inspect`]).
//! * [`AuctionFeaturedCard`]     — the live featured auction card. The
//!   card_id lives on the `ShopAuctionAuctionState` resource, not on the
//!   entity itself, so the producer looks up the active card off the
//!   resource when the press lands on the featured-card root.
//!
//! Behaviour mirrors the hand / draft consumer: secondary-button press
//! opens the overlay; Escape, secondary-button-on-same-card, or a click
//! on the dim backdrop dismisses it. Primary-button presses are ignored
//! so existing shop / auction click interactions (purchase, bid, pass)
//! are unaffected.
//!
//! Hand / draft / shared protocol / server are explicitly out of scope
//! for this consumer; the read-only import of
//! [`crate::ui::hand::inspect::build_card_inspect_view_from_card`] is the
//! single intentional cross-module reference (pure `CardData →
//! CardInspectView` mapping, no behavioural coupling).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use shared::card::CardId;

use crate::ui::card_inspect::spawn_card_inspect;
use crate::ui::design_tokens::z_layers;
use crate::ui::hand::inspect::build_card_inspect_view_from_card;

use super::{
    AuctionFeaturedCard, DraftInitialSlotCard, ShopAuctionAuctionState, ShopAuctionCardCatalog,
    ShopSlotCard,
};

/// Currently inspected shop/auction card. `None` means the overlay is closed.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct ShopAuctionCardInspectTarget(pub Option<CardId>);

/// Request to open the shop/auction inspect overlay for `card_id`.
/// Re-requesting the same id toggles the overlay closed so right-click
/// on the same card is its own dismiss gesture.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionCardInspectRequested {
    pub card_id: CardId,
}

/// Request to close the shop/auction inspect overlay regardless of
/// current target.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShopAuctionCardInspectDismissed;

/// Marks the absolute-positioned overlay root that owns the spawned
/// `card_inspect` primitive plus the click-to-dismiss backdrop.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionCardInspectOverlayRoot;

/// Reads buffered `Pointer<Press>` messages and emits
/// [`ShopAuctionCardInspectRequested`] when a secondary-button press
/// lands on a shop/auction card surface:
///
/// * [`ShopSlotCard`]         — emits the slot's `CardId`.
/// * [`DraftInitialSlotCard`] — emits the slot's `CardId`.
/// * [`AuctionFeaturedCard`]  — emits `ShopAuctionAuctionState.card_id`
///   when present (the featured-card entity does not carry the id).
///
/// Primary-button presses are ignored so existing purchase / bid /
/// pass interactions remain unaffected.
pub fn produce_shop_auction_card_inspect_requests_system(
    mut presses: MessageReader<Pointer<Press>>,
    shop_slots: Query<&ShopSlotCard>,
    draft_slots: Query<&DraftInitialSlotCard>,
    featured: Query<(), With<AuctionFeaturedCard>>,
    auction_state: Res<ShopAuctionAuctionState>,
    mut writer: MessageWriter<ShopAuctionCardInspectRequested>,
) {
    for press in presses.read() {
        if press.button != PointerButton::Secondary {
            continue;
        }
        if let Ok(slot) = shop_slots.get(press.entity) {
            writer.write(ShopAuctionCardInspectRequested { card_id: slot.0 });
            continue;
        }
        if let Ok(slot) = draft_slots.get(press.entity) {
            writer.write(ShopAuctionCardInspectRequested { card_id: slot.0 });
            continue;
        }
        if featured.get(press.entity).is_ok() {
            if let Some(card_id) = auction_state.card_id {
                writer.write(ShopAuctionCardInspectRequested { card_id });
            }
        }
    }
}

/// Folds the latest [`ShopAuctionCardInspectRequested`] and any dismiss
/// signal (explicit message, Escape key) into the
/// [`ShopAuctionCardInspectTarget`] resource. Re-requesting the currently
/// inspected card toggles it closed; requesting a different card switches
/// directly without an intermediate dismiss.
pub fn apply_shop_auction_card_inspect_target_system(
    mut requested: MessageReader<ShopAuctionCardInspectRequested>,
    mut dismissed: MessageReader<ShopAuctionCardInspectDismissed>,
    keys: Res<ButtonInput<KeyCode>>,
    mut target: ResMut<ShopAuctionCardInspectTarget>,
) {
    let latest = requested.read().last().map(|r| r.card_id);
    let mut dismiss = false;
    for _ in dismissed.read() {
        dismiss = true;
    }
    if keys.just_pressed(KeyCode::Escape) {
        dismiss = true;
    }

    if let Some(card_id) = latest {
        if target.0 == Some(card_id) {
            target.0 = None;
        } else {
            target.0 = Some(card_id);
        }
    } else if dismiss && target.0.is_some() {
        target.0 = None;
    }
}

/// Spawn / despawn the overlay tree to match
/// [`ShopAuctionCardInspectTarget`]. Only runs on resource change so the
/// steady state has zero per-frame allocation.
pub fn sync_shop_auction_card_inspect_overlay_system(
    mut commands: Commands,
    target: Res<ShopAuctionCardInspectTarget>,
    catalog: Res<ShopAuctionCardCatalog>,
    overlays: Query<Entity, With<ShopAuctionCardInspectOverlayRoot>>,
) {
    if !target.is_changed() {
        return;
    }

    for entity in &overlays {
        commands.entity(entity).despawn();
    }

    let Some(card_id) = target.0 else {
        return;
    };
    let Some(data) = catalog.cards.get(&card_id) else {
        return;
    };

    let view = build_card_inspect_view_from_card(data);

    commands
        .spawn((
            ShopAuctionCardInspectOverlayRoot,
            Name::new("Shop Auction Card Inspect Overlay"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(z_layers::MODAL.0),
            FocusPolicy::Block,
            Interaction::default(),
        ))
        .with_children(|parent| {
            spawn_card_inspect(parent, view);
        });
}

/// Click the dimmed backdrop to dismiss. The inner `card_inspect` tree
/// blocks focus locally so clicks on the card itself do not bubble back
/// here.
pub fn handle_shop_auction_card_inspect_backdrop_dismiss_system(
    interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<ShopAuctionCardInspectOverlayRoot>),
    >,
    mut writer: MessageWriter<ShopAuctionCardInspectDismissed>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            writer.write(ShopAuctionCardInspectDismissed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<ShopAuctionCardInspectTarget>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<ShopAuctionCardInspectRequested>()
            .add_message::<ShopAuctionCardInspectDismissed>()
            .add_systems(Update, apply_shop_auction_card_inspect_target_system);
        app
    }

    #[test]
    fn request_opens_then_repeat_request_closes() {
        let mut app = make_test_app();
        app.world_mut()
            .resource_mut::<Messages<ShopAuctionCardInspectRequested>>()
            .write(ShopAuctionCardInspectRequested {
                card_id: CardId(401),
            });
        app.update();
        assert_eq!(
            app.world().resource::<ShopAuctionCardInspectTarget>().0,
            Some(CardId(401))
        );

        app.world_mut()
            .resource_mut::<Messages<ShopAuctionCardInspectRequested>>()
            .write(ShopAuctionCardInspectRequested {
                card_id: CardId(401),
            });
        app.update();
        assert_eq!(
            app.world().resource::<ShopAuctionCardInspectTarget>().0,
            None
        );
    }

    #[test]
    fn dismiss_message_closes_overlay() {
        let mut app = make_test_app();
        app.world_mut()
            .resource_mut::<ShopAuctionCardInspectTarget>()
            .0 = Some(CardId(402));
        app.world_mut()
            .resource_mut::<Messages<ShopAuctionCardInspectDismissed>>()
            .write(ShopAuctionCardInspectDismissed);
        app.update();
        assert_eq!(
            app.world().resource::<ShopAuctionCardInspectTarget>().0,
            None
        );
    }

    #[test]
    fn request_switches_to_different_card_without_dismiss() {
        let mut app = make_test_app();
        app.world_mut()
            .resource_mut::<ShopAuctionCardInspectTarget>()
            .0 = Some(CardId(401));
        app.world_mut()
            .resource_mut::<Messages<ShopAuctionCardInspectRequested>>()
            .write(ShopAuctionCardInspectRequested {
                card_id: CardId(402),
            });
        app.update();
        assert_eq!(
            app.world().resource::<ShopAuctionCardInspectTarget>().0,
            Some(CardId(402))
        );
    }
}
