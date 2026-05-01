# Bevy 0.18 — Current Best Practices

Last verified: 2026-04-28

Patterns that are correct in 0.18 but that an LLM trained on older Bevy may not know.

---

## Spawning Entities (Required Components pattern)

```rust
// The correct 0.18 pattern: only insert what you need.
// Required components are automatically added by the engine.

// Spawn a 2D sprite (unit on the board)
commands.spawn((
    Sprite::from_image(texture_handle),
    Transform::from_xyz(lane_x, cell_y, z_layer),
    // DO NOT add: GlobalTransform, Visibility, InheritedVisibility (auto-required)
));

// Spawn a UI text label (gold display)
commands.spawn((
    Text::new("10"),
    TextFont { font: font_handle.clone(), font_size: 20. },
    TextColor(Color::WHITE),
    Node { position_type: PositionType::Absolute, ..default() },
    // LineHeight is required by Text — auto-inserted with a default value
    // Override explicitly if needed:
    LineHeight::RelativePx(1.5),
));

// Spawn a 2D camera
commands.spawn((
    Camera2d,
    Transform::from_xyz(0., 0., 999.),
));
```

---

## Systems Returning Results

```rust
// 0.16+ pattern — systems can return BevyError
fn resolve_combat(
    mut units: Query<(&mut Stats, &Position)>,
    mut objectives: Query<&mut Objective>,
) -> Result<(), BevyError> {
    let (mut stats, pos) = units.single_mut()?;  // returns Err if 0 or 2+ matches
    // ...
    Ok(())
}
```

---

## Messages vs Observers (0.17+)

```rust
// ⚠️ EventWriter/EventReader DO NOT EXIST in Bevy 0.17+.
// Use MessageWriter/MessageReader for BUFFERED game-loop messages:
// — placement submitted, bid placed, gold awarded
// These are buffered and read via MessageReader each frame.

#[derive(Message, Clone, Debug)]
struct UnitPlaced { lane: u8, card: CardId }

// Register: app.add_message::<UnitPlaced>();

fn emit_placement(mut writer: MessageWriter<UnitPlaced>, /* ... */) {
    writer.write(UnitPlaced { lane: 2, card: CardId(42) });
}

fn handle_placement(mut reader: MessageReader<UnitPlaced>, /* ... */) {
    for msg in reader.read() { /* ... */ }
}

// Use Observers for REACTIVE triggers that fire immediately:
// — APPEARANCE, DEATH, FINAL BLOW keywords
// These fire synchronously when the triggering action occurs.

#[derive(Event)]
struct UnitDied { attacker: Entity }

commands.entity(unit_entity).observe(|trigger: On<UnitDied>, /* ... */| {
    // DEATH trigger — fires immediately when unit HP reaches 0
});
```

---

## UI Hierarchy (bevy_ui 0.18)

```rust
// Card in hand — correct 0.18 pattern
fn spawn_card(commands: &mut Commands, card: &CardData, pos: Vec2, assets: &GameAssets) {
    commands.spawn((
        // Background node
        Node {
            width: Val::Px(120.),
            height: Val::Px(180.),
            position_type: PositionType::Absolute,
            left: Val::Px(pos.x),
            bottom: Val::Px(pos.y),
            border_radius: BorderRadius::all(Val::Px(8.)),  // 0.18: field in Node
            ..default()
        },
        ImageNode::new(assets.card_frame.clone()),  // 0.16+: was UiImage
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(&card.name),
            TextFont { font: assets.font.clone(), font_size: 14. },
            TextColor(Color::WHITE),
            Node { margin: UiRect::all(Val::Px(4.)), ..default() },
        ));
    });
}
```

---

## Lightyear 0.26 Patterns (Bevy 0.18)

```rust
// Protocol definition — shared between client and server
#[derive(Message, Serialize, Deserialize, Clone)]
pub struct C2SPlaceUnit {
    pub lane: u8,
    pub card_id: CardId,
}

#[derive(Message, Serialize, Deserialize, Clone)]
pub struct S2CRoundResolved {
    pub events: Vec<CombatEvent>,
    pub new_gold: HashMap<ClientId, u32>,
}

// Sending a message from client to server
fn submit_placement(
    mut sender: MessageSender<C2SPlaceUnit>,
    // ...
) {
    sender.send_to_server(C2SPlaceUnit { lane: 2, card_id: selected_card });
}

// Receiving on server
fn handle_placement(
    mut receiver: MessageReceiver<C2SPlaceUnit>,
    // ...
) {
    for (client_id, msg) in receiver.receive_messages() {
        // validate and record placement
    }
}

// LocalTimeline is now a Resource (0.26 change)
// NetworkVisibility methods → ReplicationState
```

---

## Cargo.toml Template (Bevy 0.18 project)

```toml
[workspace]
members = ["client", "server", "shared"]

# shared/Cargo.toml
[dependencies]
bevy = { version = "0.18", default-features = false, features = ["serialize"] }
lightyear = { version = "0.26", features = ["shared"] }
serde = { version = "1", features = ["derive"] }

# client/Cargo.toml
[dependencies]
shared = { path = "../shared" }
bevy = { version = "0.18", features = [
    "bevy_ui", "bevy_sprite", "bevy_text",
    "bevy_asset", "bevy_audio",
    "mouse", "keyboard",
    "webgl2",          # for WASM
] }
lightyear = { version = "0.26", features = ["client", "websocket"] }
bevy_tweening = "0.18"
bevy_asset_loader = "0.22"  # verify compatible version on crates.io

# server/Cargo.toml
[dependencies]
shared = { path = "../shared" }
bevy = { version = "0.18", default-features = false, features = ["multi_threaded"] }
lightyear = { version = "0.26", features = ["server", "websocket"] }
rand = "0.9"
rand_chacha = "0.3"
```

---

## WASM Build Setup

```toml
# client/Trunk.toml
[build]
target = "index.html"

[watch]
ignore = ["../server", "../shared/src/server*"]
```

```html
<!-- client/index.html — minimal WASM host -->
<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Lanes and Lies</title></head>
<body style="margin:0;background:#000">
  <canvas id="bevy_canvas"></canvas>
</body>
</html>
```

```bash
# Build commands
trunk serve                        # dev server (hot reload)
trunk build --release              # production WASM bundle → dist/
cargo build --release -p server    # production server binary
```
