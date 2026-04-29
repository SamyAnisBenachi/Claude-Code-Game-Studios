# Bevy Breaking Changes — 0.14 → 0.18

Last verified: 2026-04-28
Source: Official Bevy migration guides

This document lists breaking changes across all four post-training-cutoff versions, filtered
for relevance to this project (ECS, 2D sprites, UI, events, asset loading, WASM/networking).

---

## Bevy 0.15 — Required Components (THE big one)

**Bundles are deprecated. Use Required Components instead.**

```rust
// OLD (pre-0.15) — SpriteBundle, TransformBundle, etc.
commands.spawn(SpriteBundle {
    texture: my_texture.clone(),
    transform: Transform::from_xyz(0., 0., 0.),
    ..default()
});

// NEW (0.15+) — just insert what you need; requirements auto-added
commands.spawn((
    Sprite::from_image(my_texture.clone()),
    Transform::from_xyz(0., 0., 0.),
    // GlobalTransform, Visibility, etc. inserted automatically
));
```

Key bundle deprecations:
- `SpriteBundle` → `Sprite` + `Transform` (GlobalTransform, Visibility auto-required)
- `TransformBundle` → `Transform` alone (GlobalTransform auto-required)
- `SpatialBundle` → `Transform` + `Visibility`
- `Camera2dBundle` → `Camera2d` + `Transform`
- `NodeBundle` → `Node` + style fields
- All rendering bundles → their primary component + Required Components

**Transform/GlobalTransform:**
```rust
// GlobalTransform is now ALWAYS auto-inserted when you insert Transform.
// Never insert GlobalTransform manually.
commands.spawn(Transform::from_xyz(x, y, z)); // GlobalTransform added automatically
```

---

## Bevy 0.16 — Query, Events, UI, Hierarchy

### Query::single() now returns Result

```rust
// OLD (panics if 0 or 2+ results)
let player = query.single();

// NEW — handle the result
let Ok(player) = query.single() else { return; };
// or propagate: systems can return Result<(), BevyError>
fn my_system(query: Query<&Transform, With<Player>>) -> Result<(), BevyError> {
    let transform = query.single()?;
    Ok(())
}
```

### EventWriter renamed

```rust
// OLD
event_writer.send(MyEvent { ... });
event_writer.send_batch(events);
event_writer.send_default();

// NEW
event_writer.write(MyEvent { ... });
event_writer.write_batch(events);
event_writer.write_default();
```

### Hierarchy: Parent → ChildOf

```rust
// OLD
commands.entity(child).set_parent(parent);
let parent_entity = *parent_component;  // Parent derefs to Entity

// NEW
commands.entity(child).insert(ChildOf(parent));
let parent_entity = child_of.parent();  // explicit method
```

### Despawn behavior changed

```rust
// OLD
commands.entity(e).despawn_recursive();       // despawns entity + all children
commands.entity(e).despawn_descendants();     // despawns only children

// NEW
commands.entity(e).despawn();                             // despawns entity + all children
commands.entity(e).despawn_related::<Children>();         // despawns only children
```

### UI: UiImage → ImageNode

```rust
// OLD
commands.spawn(UiImage::new(texture_handle));

// NEW
commands.spawn(ImageNode::new(texture_handle));
// UiImageSize → ImageNodeSize
```

---

## Bevy 0.17 — Event/Observer Split, Render Reorganization

### Event vs Message: critical distinction

```rust
// ⚠️ BEVY 0.17+ BREAKING CHANGE — EventWriter/EventReader REMOVED
//
// Old names (pre-0.17): EventWriter<T>, EventReader<T>, Events<T>
// These types DO NOT EXIST in Bevy 0.17+.
//
// NEW API — two distinct mechanisms:
//
// 1. BUFFERED MESSAGES (pull-based, polled each frame):
//    #[derive(Message)] + MessageWriter<T> + MessageReader<T> + app.add_message::<T>()
#[derive(Message)]
struct UnitPlaced { lane: u8 }
// fn emit(mut w: MessageWriter<UnitPlaced>) { w.write(UnitPlaced { lane: 0 }); }
// fn read(mut r: MessageReader<UnitPlaced>) { for msg in r.read() { ... } }
//
// 2. OBSERVER EVENTS (push-based, immediate/same-frame trigger):
//    #[derive(Event)] + commands.trigger() / trigger_targets() + Observer
#[derive(Event)]
struct UnitDied { entity: Entity }
// commands.trigger_targets(UnitDied { entity }, target_entity);
// app.observe(|t: On<UnitDied>| { ... });
//
// See liv-bevy-018 skill for full patterns.
```

### bevy_render reorganization

Several render types moved to new sub-crates. If you import from `bevy::render::*`,
verify paths in 0.17 — some types moved. For this project (2D sprites, UI, no custom
render pipelines), impact is minimal.

---

## Bevy 0.18 — UI, Text, Entities, Input Features

### Text: LineHeight is now a required component

```rust
// OLD — TextFont had line_height field
commands.spawn(TextFont {
    font: ...,
    font_size: 24.,
    line_height: 1.2,  // ← REMOVED
});

// NEW — LineHeight is a separate required component
commands.spawn((
    Text::new("Hello"),
    TextFont { font: ..., font_size: 24. },
    LineHeight::RelativePx(1.2),  // separate component, auto-required
));
// Text, Text2d, and TextSpan all require LineHeight as of 0.18
```

### BorderRadius moved into Node

```rust
// OLD — BorderRadius was a separate component
commands.spawn((Node { .. }, BorderRadius::all(Val::Px(8.))));

// NEW — BorderRadius is a field inside Node
commands.spawn(Node {
    border_radius: BorderRadius::all(Val::Px(8.)),
    ..default()
});
```

### Camera: RenderTarget is now a required component

```rust
// If you need a custom render target, insert it explicitly.
// For the default window target: no change needed — it's auto-required.
// Only affects code that manually set camera.target = RenderTarget::...
```

### Entity API: EntityRow → EntityIndex

```rust
// OLD
entity.row()

// NEW
entity.index()
// EntityRow type → EntityIndex
// Entities::flush() removed — use World::spawn() or EntityRows directly
```

### Input is now behind cargo features

```rust
// In Cargo.toml, bevy's default features include mouse/keyboard/gamepad.
// If you use a custom feature set, add explicitly:
bevy = { version = "0.18", features = ["bevy_ui", "mouse", "keyboard"] }
// For WASM: touch may be needed for mobile browsers
```

### AssetLoader now requires TypePath

```rust
// Any custom AssetLoader impl must now derive TypePath:
#[derive(Default, TypePath)]
struct MyLoader;

impl AssetLoader for MyLoader { ... }
```

### ron no longer re-exported from bevy_scene / bevy_asset

```rust
// Add ron as a direct dependency in Cargo.toml:
// [dependencies]
// ron = "0.8"
```

### Reflect attribute syntax: only parentheses

```rust
// OLD (also accepted brackets and braces)
#[reflect[Debug, PartialEq]]
#[reflect{Clone}]

// NEW — parentheses only
#[reflect(Debug, PartialEq, Clone)]
```

### Feature renames (Cargo.toml)

| Old feature name | New feature name |
|---|---|
| `animation` | `gltf_animation` |
| `bevy_sprite_picking_backend` | `sprite_picking` |
| `bevy_ui_picking_backend` | `ui_picking` |
| `bevy_mesh_picking_backend` | `mesh_picking` |
