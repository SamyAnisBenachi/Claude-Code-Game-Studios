//! Hand / draft card inspect overlay consumer.
//!
//! Wires the shared `card_inspect` primitive (PROMPT 1482) into the hand fan
//! and the DRAFT_INITIAL grid so the local player can right-click a card to
//! open an enlarged view, then dismiss with Escape, right-click, or by
//! clicking the dimmed backdrop.
//!
//! Shop / auction surfaces are owned by `client/src/ui/shop_auction/**` and
//! are explicitly out of scope here (PROMPT 1520 forbidden path).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use shared::card::{CardData, CardId, CardType, Keyword, SimpleKeyword};

use crate::ui::card_inspect::{spawn_card_inspect, CardInspectView};
use crate::ui::design_tokens::z_layers;

use super::{GridSlotCard, HandCardCatalog, HandSlotCard};

/// Currently inspected card. `None` means the overlay is closed.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct HandCardInspectTarget(pub Option<CardId>);

/// Request to open inspect for `card_id`. Re-requesting the same id toggles
/// the overlay closed so right-click is its own dismiss gesture on the same
/// card.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandCardInspectRequested {
    pub card_id: CardId,
}

/// Request to close the inspect overlay regardless of current target.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HandCardInspectDismissed;

/// Marks the absolute-positioned overlay root that owns the spawned
/// `card_inspect` primitive plus the click-to-dismiss backdrop.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HandCardInspectOverlayRoot;

/// Reads buffered `Pointer<Press>` messages and emits
/// [`HandCardInspectRequested`] when a secondary-button press lands on a
/// `HandSlotCard` (fan) or `GridSlotCard` (DRAFT_INITIAL grid). Primary-button
/// presses are ignored so the existing placement-drag producer is unaffected.
pub fn produce_hand_card_inspect_requests_system(
    mut presses: MessageReader<Pointer<Press>>,
    fan_slots: Query<&HandSlotCard>,
    grid_slots: Query<&GridSlotCard>,
    mut writer: MessageWriter<HandCardInspectRequested>,
) {
    for press in presses.read() {
        if press.button != PointerButton::Secondary {
            continue;
        }
        if let Ok(slot) = fan_slots.get(press.entity) {
            writer.write(HandCardInspectRequested { card_id: slot.0 });
            continue;
        }
        if let Ok(slot) = grid_slots.get(press.entity) {
            writer.write(HandCardInspectRequested { card_id: slot.0 });
        }
    }
}

/// Folds the latest `HandCardInspectRequested` and any dismiss signal
/// (explicit message, Escape key) into the `HandCardInspectTarget` resource.
/// Re-requesting the currently-inspected card toggles it closed.
pub fn apply_hand_card_inspect_target_system(
    mut requested: MessageReader<HandCardInspectRequested>,
    mut dismissed: MessageReader<HandCardInspectDismissed>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut target: ResMut<HandCardInspectTarget>,
) {
    let latest = requested.read().last().map(|r| r.card_id);
    let mut dismiss = false;
    for _ in dismissed.read() {
        dismiss = true;
    }
    if let Some(keys) = keys.as_deref() {
        if keys.just_pressed(KeyCode::Escape) {
            dismiss = true;
        }
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

/// Sync: spawn / despawn the overlay tree to match
/// [`HandCardInspectTarget`]. Only runs on resource change so the steady
/// state has zero per-frame allocation.
pub fn sync_hand_card_inspect_overlay_system(
    mut commands: Commands,
    target: Res<HandCardInspectTarget>,
    catalog: Res<HandCardCatalog>,
    overlays: Query<Entity, With<HandCardInspectOverlayRoot>>,
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
            HandCardInspectOverlayRoot,
            Name::new("Hand Card Inspect Overlay"),
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

/// Click the dimmed backdrop to dismiss. The inner card_inspect tree blocks
/// focus locally so clicks on the card itself do not bubble back here.
pub fn handle_hand_card_inspect_backdrop_dismiss_system(
    interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<HandCardInspectOverlayRoot>),
    >,
    mut writer: MessageWriter<HandCardInspectDismissed>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            writer.write(HandCardInspectDismissed);
        }
    }
}

/// Projects a [`CardData`] into the primitive's [`CardInspectView`]. Public so
/// focused tests can assert the mapping without spinning up the full overlay
/// spawn path.
pub fn build_card_inspect_view_from_card(data: &CardData) -> CardInspectView {
    let title = if !data.name_en.is_empty() {
        data.name_en.clone()
    } else if !data.name_fr.is_empty() {
        data.name_fr.clone()
    } else {
        format!("Card #{}", data.id.0)
    };

    let (attack, health) = match data.card_type {
        CardType::Minion | CardType::Structure => (
            Some(data.atk.to_string()),
            Some(data.hp.to_string()),
        ),
        _ => (None, None),
    };

    let keyword = if data.keywords.is_empty() {
        None
    } else {
        Some(format_keywords(&data.keywords))
    };

    let keyword_glossary: Vec<(String, String)> = data
        .keywords
        .iter()
        .map(|kw| (format_keyword(kw), keyword_glossary_definition(kw)))
        .collect();

    let rules_text = if data.effect_text.trim().is_empty() {
        "No card text.".to_string()
    } else {
        data.effect_text.clone()
    };

    CardInspectView {
        title,
        cost: Some(data.cost.to_string()),
        attack,
        health,
        keyword,
        keyword_glossary,
        rules_text,
    }
}

/// Returns a short player-readable definition for a keyword variant.
/// Used to populate the glossary panel in the inspect overlay.
pub fn keyword_glossary_definition(keyword: &Keyword) -> String {
    match keyword {
        Keyword::Simple(simple) => simple_keyword_definition(*simple).to_string(),
        Keyword::RangeX { max_range } => {
            format!("Attacks enemies up to {max_range} cells away.")
        }
        Keyword::ChargeXMove { cells } => {
            format!("Can move up to {cells} extra cells per activation.")
        }
        Keyword::ResistanceX { value } => format!("Reduces incoming damage by {value}."),
        Keyword::VulnerabilityX { value } => format!("Increases incoming damage by {value}."),
        Keyword::RepelX { distance } => format!("Pushes the target up to {distance} cells away."),
        Keyword::AttractX { distance } => {
            format!("Pulls the target up to {distance} cells closer.")
        }
    }
}

fn simple_keyword_definition(keyword: SimpleKeyword) -> &'static str {
    match keyword {
        SimpleKeyword::Appearance => "Triggers when this unit enters the board.",
        SimpleKeyword::Death => "Triggers when this unit dies.",
        SimpleKeyword::FinalBlow => "Triggers when this unit scores the killing blow.",
        SimpleKeyword::Counterattack => "Retaliates when damaged by an attacker.",
        SimpleKeyword::StartOfTurn => "Triggers at the start of your turn.",
        SimpleKeyword::EndOfTurn => "Triggers at the end of your turn.",
        SimpleKeyword::FirstStrike => "Deals damage before normal units in combat.",
        SimpleKeyword::Haste => "Can attack the turn it is summoned.",
        SimpleKeyword::Wall => "Cannot move or be pushed past the center line.",
        SimpleKeyword::Bodyguard => "Nearby allies cannot be targeted while this unit is alive.",
        SimpleKeyword::Irremovable => "Cannot be displaced by push, pull, or teleport.",
        SimpleKeyword::Untargetable => "Cannot be directly targeted by spells or abilities.",
        SimpleKeyword::Shield => "Blocks the next source of damage, then consumed.",
        SimpleKeyword::Leader => "Gains the stats of the weakest enemy in this lane.",
        SimpleKeyword::Outnumbered => "Gains a bonus when outnumbered in this lane.",
        SimpleKeyword::ArmorPiercing => "Ignores enemy Resistance when dealing damage.",
        SimpleKeyword::Silence => "Removes all keyword abilities from the target.",
        SimpleKeyword::Stun => "Prevents the target from acting this round.",
        SimpleKeyword::Teleport => "Moves this unit to any empty cell on the board.",
        SimpleKeyword::ChangeLane => "This unit can move to an adjacent lane.",
    }
}

fn format_keywords(keywords: &[Keyword]) -> String {
    keywords
        .iter()
        .map(format_keyword)
        .collect::<Vec<_>>()
        .join(" · ")
}

fn format_keyword(keyword: &Keyword) -> String {
    match keyword {
        Keyword::Simple(simple) => format_simple_keyword(*simple).to_string(),
        Keyword::RangeX { max_range } => format!("Range {}", max_range),
        Keyword::ChargeXMove { cells } => format!("Charge {}", cells),
        Keyword::ResistanceX { value } => format!("Resistance {}", value),
        Keyword::VulnerabilityX { value } => format!("Vulnerability {}", value),
        Keyword::RepelX { distance } => format!("Repel {}", distance),
        Keyword::AttractX { distance } => format!("Attract {}", distance),
    }
}

fn format_simple_keyword(keyword: SimpleKeyword) -> &'static str {
    match keyword {
        SimpleKeyword::Appearance => "Appearance",
        SimpleKeyword::Death => "Death",
        SimpleKeyword::FinalBlow => "Final Blow",
        SimpleKeyword::Counterattack => "Counterattack",
        SimpleKeyword::StartOfTurn => "Start of Turn",
        SimpleKeyword::EndOfTurn => "End of Turn",
        SimpleKeyword::FirstStrike => "First Strike",
        SimpleKeyword::Haste => "Haste",
        SimpleKeyword::Wall => "Wall",
        SimpleKeyword::Bodyguard => "Bodyguard",
        SimpleKeyword::Irremovable => "Irremovable",
        SimpleKeyword::Untargetable => "Untargetable",
        SimpleKeyword::Shield => "Shield",
        SimpleKeyword::Leader => "Leader",
        SimpleKeyword::Outnumbered => "Outnumbered",
        SimpleKeyword::ArmorPiercing => "Armor Piercing",
        SimpleKeyword::Silence => "Silence",
        SimpleKeyword::Stun => "Stun",
        SimpleKeyword::Teleport => "Teleport",
        SimpleKeyword::ChangeLane => "Change Lane",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::card::{ClassId, Rarity, UnitType};

    fn minion_fixture() -> CardData {
        CardData {
            id: CardId(101),
            name_fr: "Bouftou".to_string(),
            name_en: "Boowolf".to_string(),
            class: ClassId::Neutral,
            family: None,
            rarity: Rarity::Common,
            card_type: CardType::Minion,
            unit_type: UnitType::Blade,
            cost: 2,
            atk: 3,
            hp: 4,
            mp: 1,
            ar: 0,
            keywords: vec![
                Keyword::Simple(SimpleKeyword::Haste),
                Keyword::ResistanceX { value: 2 },
            ],
            effect_text: "Charges into the lane.".to_string(),
            art_id: "art_boowolf".to_string(),
            pool_copies_override: None,
        }
    }

    fn spell_fixture() -> CardData {
        CardData {
            id: CardId(202),
            name_fr: "".to_string(),
            name_en: "Bolt".to_string(),
            class: ClassId::Cra,
            family: None,
            rarity: Rarity::Uncommon,
            card_type: CardType::Spell,
            unit_type: UnitType::Neutral,
            cost: 1,
            atk: 0,
            hp: 0,
            mp: 0,
            ar: 0,
            keywords: vec![],
            effect_text: "".to_string(),
            art_id: "art_bolt".to_string(),
            pool_copies_override: None,
        }
    }

    #[test]
    fn build_view_minion_includes_attack_health_keywords() {
        let view = build_card_inspect_view_from_card(&minion_fixture());
        assert_eq!(view.title, "Boowolf");
        assert_eq!(view.cost.as_deref(), Some("2"));
        assert_eq!(view.attack.as_deref(), Some("3"));
        assert_eq!(view.health.as_deref(), Some("4"));
        let keyword = view.keyword.expect("keyword line");
        assert!(keyword.contains("Haste"));
        assert!(keyword.contains("Resistance 2"));
        assert_eq!(view.rules_text, "Charges into the lane.");
    }

    #[test]
    fn glossary_entries_non_empty_for_keyworded_minion() {
        let view = build_card_inspect_view_from_card(&minion_fixture());
        // Minion fixture has Haste + ResistanceX{2} — must produce 2 glossary entries.
        assert_eq!(
            view.keyword_glossary.len(),
            2,
            "expected one glossary entry per keyword"
        );
        let (haste_label, haste_def) = &view.keyword_glossary[0];
        assert_eq!(haste_label, "Haste");
        assert!(!haste_def.is_empty(), "Haste definition must not be empty");

        let (resist_label, resist_def) = &view.keyword_glossary[1];
        assert_eq!(resist_label, "Resistance 2");
        assert!(
            !resist_def.is_empty(),
            "ResistanceX definition must not be empty"
        );
        assert!(
            resist_def.contains('2'),
            "ResistanceX definition should include the value"
        );
    }

    #[test]
    fn glossary_empty_for_keyword_free_card() {
        let view = build_card_inspect_view_from_card(&spell_fixture());
        assert!(
            view.keyword_glossary.is_empty(),
            "card with no keywords must have empty glossary"
        );
    }

    #[test]
    fn build_view_spell_omits_attack_health_and_fills_fallback_rules_text() {
        let view = build_card_inspect_view_from_card(&spell_fixture());
        assert_eq!(view.title, "Bolt");
        assert_eq!(view.cost.as_deref(), Some("1"));
        assert!(view.attack.is_none());
        assert!(view.health.is_none());
        assert!(view.keyword.is_none());
        assert_eq!(view.rules_text, "No card text.");
    }

    fn make_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<HandCardInspectTarget>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<HandCardInspectRequested>()
            .add_message::<HandCardInspectDismissed>()
            .add_systems(Update, apply_hand_card_inspect_target_system);
        app
    }

    #[test]
    fn request_opens_then_repeat_request_closes() {
        let mut app = make_test_app();
        app.world_mut()
            .resource_mut::<Messages<HandCardInspectRequested>>()
            .write(HandCardInspectRequested {
                card_id: CardId(101),
            });
        app.update();
        assert_eq!(
            app.world().resource::<HandCardInspectTarget>().0,
            Some(CardId(101))
        );

        app.world_mut()
            .resource_mut::<Messages<HandCardInspectRequested>>()
            .write(HandCardInspectRequested {
                card_id: CardId(101),
            });
        app.update();
        assert_eq!(app.world().resource::<HandCardInspectTarget>().0, None);
    }

    #[test]
    fn dismiss_message_closes_overlay() {
        let mut app = make_test_app();
        app.world_mut().resource_mut::<HandCardInspectTarget>().0 = Some(CardId(202));
        app.world_mut()
            .resource_mut::<Messages<HandCardInspectDismissed>>()
            .write(HandCardInspectDismissed);
        app.update();
        assert_eq!(app.world().resource::<HandCardInspectTarget>().0, None);
    }

    #[test]
    fn apply_target_system_runs_without_button_input_resource() {
        let mut app = App::new();
        app.init_resource::<HandCardInspectTarget>()
            .add_message::<HandCardInspectRequested>()
            .add_message::<HandCardInspectDismissed>()
            .add_systems(Update, apply_hand_card_inspect_target_system);

        app.world_mut()
            .resource_mut::<Messages<HandCardInspectRequested>>()
            .write(HandCardInspectRequested {
                card_id: CardId(303),
            });
        app.update();
        assert_eq!(
            app.world().resource::<HandCardInspectTarget>().0,
            Some(CardId(303))
        );
    }

    #[test]
    fn request_switches_to_different_card_without_dismiss() {
        let mut app = make_test_app();
        app.world_mut().resource_mut::<HandCardInspectTarget>().0 = Some(CardId(101));
        app.world_mut()
            .resource_mut::<Messages<HandCardInspectRequested>>()
            .write(HandCardInspectRequested {
                card_id: CardId(202),
            });
        app.update();
        assert_eq!(
            app.world().resource::<HandCardInspectTarget>().0,
            Some(CardId(202))
        );
    }
}
