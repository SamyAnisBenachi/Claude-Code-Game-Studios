use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::{prelude::*, time::Virtual};
use bevy_tweening::TweeningPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::design_tokens::strips::HandBar;
use client::ui::hand::{HandCardCatalog, HandFanRoot, HandUiPlugin, HandUiTimingConfig};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// S17-UI-HAND-B0004-CLEANUP-001 AC4 — hierarchy invariant assertion.
//
// Bevy 0.18's hierarchy lint `B0004` fires when an entity carrying
// `GlobalTransform` has a `ChildOf` parent that does NOT carry
// `GlobalTransform`. The Hand UI tree spawns `HandFanRoot` (with an
// explicit `Transform`/`GlobalTransform` for fan-layout queries) as a
// child of the `strips::HandBar` strip. `bevy_ui` `Node` requires
// `UiTransform` but NOT `Transform`/`GlobalTransform` (verified against
// `bevy_ui-0.18.1` `src/ui_node.rs` `Node` `#[require(...)]` set), so
// the audit-time `HandBar` carried `Node` only and the `B0004` warning
// fired on every `InSession` entry (PROMPT 1076 `AUDIT-1076-14`).
//
// Strategy A: insert `Transform::default()` on `HandBar`. The Bevy 0.18
// Required Components API auto-derives `GlobalTransform` from `Transform`
// (verified against `bevy_transform-0.18.1` `src/components/transform.rs`
// `Transform` `require(GlobalTransform, TransformTreeChanged)`), so the
// invariant holds without any explicit `GlobalTransform` insert.
//
// This test asserts:
//   (a) `HandFanRoot` carries `GlobalTransform` (precondition for the
//       audit invariant — confirms the spawn site still meets the
//       child side of the `B0004` shape).
//   (b) `HandFanRoot`'s `ChildOf` parent (the `HandBar` strip) carries
//       `GlobalTransform`.
//   (c) The parent also carries the `HandBar` marker component, so the
//       assertion shape locks the precise relationship the audit
//       called out ("Hand UI Fan Root's parent HandBar lacks
//       GlobalTransform") rather than passing on any arbitrary
//       Transform-carrying ancestor.
//
// This row is ECS hierarchy hygiene only. It does NOT close the Sprint
// 12 story 019 drag-runtime `closed-with-conditions / cannot-reproduce`
// disposition, the Sprint 11 story 018 retest question, any
// `AUDIT-1076-*` finding outside `AUDIT-1076-14`, any `SOURCE-1077-*`
// finding, any of the 24 PROMPT 1022 audit findings, `S8-QA-001-W1`,
// `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, the PROMPT 761
// Polish->Release gate-check `FAIL`, or any release-readiness /
// accessibility / playtest / final-art / stage-advance claim.

#[test]
fn hand_fan_root_parent_carries_global_transform() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui();

    let fan_root = single_entity_with::<HandFanRoot>(&mut app)
        .expect("HandFanRoot must be spawned after entering InSession");

    let fan_root_has_global = app.world().get::<GlobalTransform>(fan_root).is_some();
    assert!(
        fan_root_has_global,
        "AC4 precondition: HandFanRoot must carry GlobalTransform so the B0004 \
         invariant ('child with GlobalTransform must have parent with GlobalTransform') \
         applies to this hierarchy edge",
    );

    let parent = app
        .world()
        .get::<ChildOf>(fan_root)
        .map(|c| c.parent())
        .expect("HandFanRoot must be parented (ChildOf) to the HandBar strip");

    let parent_has_handbar = app.world().get::<HandBar>(parent).is_some();
    assert!(
        parent_has_handbar,
        "AC4 specificity: HandFanRoot's parent must carry the HandBar marker so this \
         assertion locks the exact `AUDIT-1076-14` hierarchy edge (parent={parent:?}). \
         A different parent would mean the audit-time hierarchy shape has drifted and \
         the regression canary needs reauthoring rather than passing silently.",
    );

    let parent_has_global = app.world().get::<GlobalTransform>(parent).is_some();
    assert!(
        parent_has_global,
        "AC2 / AC4: HandFanRoot's parent ({parent:?}) must carry GlobalTransform so Bevy \
         0.18 does NOT emit the B0004 hierarchy warning. Strategy A repairs this by \
         inserting `Transform::default()` on the HandBar entity; the Required Components \
         API auto-derives `GlobalTransform`. If this assertion fires, either the \
         `Transform` insert on the HandBar spawn site in `client/src/ui/hand/mod.rs` was \
         removed, or Bevy's Required Components contract for `Transform` -> \
         `GlobalTransform` regressed (see `bevy_transform-0.18.1` \
         `src/components/transform.rs`).",
    );

    let parent_has_transform = app.world().get::<Transform>(parent).is_some();
    assert!(
        parent_has_transform,
        "AC2 / Strategy A: HandFanRoot's parent ({parent:?}) must carry Transform so the \
         Required Components API derives GlobalTransform. A missing Transform here means \
         the spawn-site insert was reverted — re-apply `Transform::default()` to the \
         `HandBar` tuple in `client/src/ui/hand/mod.rs::spawn_hand_ui`.",
    );
}

fn single_entity_with<C: Component>(app: &mut App) -> Option<Entity> {
    let mut query = app.world_mut().query_filtered::<Entity, With<C>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    match entities.as_slice() {
        [] => None,
        [single] => Some(*single),
        many => panic!(
            "expected a single entity with the requested marker after InSession entry; \
             found {} entities: {many:?}. The Hand UI tree should spawn exactly one \
             HandFanRoot per session.",
            many.len(),
        ),
    }
}

fn app_with_hand_ui() -> App {
    let mut app = base_app();
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(1280, 720),
            ..default()
        },
        PrimaryWindow,
    ));
    finalize_app(&mut app);
    app
}

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(TweeningPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog::default());
    app.insert_resource(PlayerEconomyView {
        gold: 5,
        reserve_mana: 0,
        initialized: true,
        ..default()
    });
    app.insert_resource(HandUiTimingConfig {
        card_draw_animation_ms: 280,
        purchase_timeout_ms: 3_000,
        hand_full_notification_duration_ms: 2_000,
    });
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app
}

fn finalize_app(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::DraftInitial;
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}
