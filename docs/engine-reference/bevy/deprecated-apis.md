# Bevy Deprecated APIs — Don't Use → Use Instead

Last verified: 2026-04-28

Quick-reference table for the most common patterns an LLM trained on pre-0.15 Bevy
will suggest incorrectly. Read this before writing any Bevy code.

---

## ECS & Spawning

| Don't use | Use instead | Since |
|---|---|---|
| `SpriteBundle { .. }` | `Sprite::from_image(..)` + `Transform` | 0.15 |
| `Camera2dBundle { .. }` | `Camera2d` + `Transform` | 0.15 |
| `NodeBundle { .. }` | `Node { .. }` | 0.15 |
| `TransformBundle { .. }` | `Transform` alone | 0.15 |
| `SpatialBundle { .. }` | `Transform` + `Visibility` | 0.15 |
| Manual `GlobalTransform` insert | Don't — auto-inserted by `Transform` | 0.15 |
| `query.single()` (panics) | `query.single()?` or `let Ok(x) = query.single()` | 0.16 |
| `query.get_single()` | `query.single()` (returns Result now) | 0.16 |
| `entity.row()` | `entity.index()` | 0.18 |
| `Entities::flush()` | Removed — use `World::spawn()` | 0.18 |
| `SimpleExecutor` | `SingleThreadedExecutor` or `MultiThreadedExecutor` | 0.18 |

## Events

| Don't use | Use instead | Since |
|---|---|---|
| `event_writer.send(e)` | `event_writer.write(e)` | 0.16 |
| `event_writer.send_batch(v)` | `event_writer.write_batch(v)` | 0.16 |
| `event_writer.send_default()` | `event_writer.write_default()` | 0.16 |

## Hierarchy / Parent-Child

| Don't use | Use instead | Since |
|---|---|---|
| `commands.entity(e).set_parent(p)` | `commands.entity(e).insert(ChildOf(p))` | 0.16 |
| `Parent` component | `ChildOf` component | 0.16 |
| `*parent_component` (deref to Entity) | `child_of.parent()` | 0.16 |
| `commands.entity(e).despawn_recursive()` | `commands.entity(e).despawn()` | 0.16 |
| `commands.entity(e).despawn_descendants()` | `commands.entity(e).despawn_related::<Children>()` | 0.16 |

## UI

| Don't use | Use instead | Since |
|---|---|---|
| `UiImage::new(handle)` | `ImageNode::new(handle)` | 0.16 |
| `UiImageSize` | `ImageNodeSize` | 0.16 |
| `TextFont { line_height: .. }` | `LineHeight` as separate component | 0.18 |
| `BorderRadius` as separate component | `Node { border_radius: .. }` field | 0.18 |
| `ExtractedUiNode::stack_index` | `ExtractedUiNode::z_order` (f32) | 0.18 |

## Reflection

| Don't use | Use instead | Since |
|---|---|---|
| `#[reflect[..]]` or `#[reflect{..}]` | `#[reflect(..)]` parentheses only | 0.18 |

## Assets

| Don't use | Use instead | Since |
|---|---|---|
| `LoadContext::path()` returning `&Path` | Returns `AssetPath` | 0.18 |
| `LoadContext::asset_path()` | Removed — use `LoadContext::path()` | 0.18 |
| `LoadContext::finish(label, meta, asset)` | `meta` param removed | 0.16 |
| `Handle::weak_from_u128(id)` | `weak_handle!` macro | 0.16 |
| `ron` from `bevy_scene` or `bevy_asset` | Add `ron = "0.8"` directly to `Cargo.toml` | 0.18 |
| `AssetLoader` without `TypePath` | Add `#[derive(TypePath)]` to loader struct | 0.18 |

## Animation / Text

| Don't use | Use instead | Since |
|---|---|---|
| `AnimationTarget { id, player }` | `AnimationTargetId` + `AnimatedBy` components | 0.18 |
| `FontAtlasSets` | Removed — font atlasing is internal | 0.18 |

## Cargo Features

| Old feature name | New name |
|---|---|
| `animation` | `gltf_animation` |
| `bevy_sprite_picking_backend` | `sprite_picking` |
| `bevy_ui_picking_backend` | `ui_picking` |
