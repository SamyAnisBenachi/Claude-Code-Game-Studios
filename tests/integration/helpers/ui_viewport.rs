//! Sprint 14 / Story 005 — viewport-invariant test helper module.
//!
//! Exposes the deterministic UI-bounds harness consumed by the integration
//! test bin at `tests/integration/ui_viewport_invariants_test.rs`. The helper
//! is split out so future Tier 1 surface stories (HUD top / bottom strip,
//! draft centered modal, lobby modal, etc.) can extend the canonical
//! viewport matrix and baseline-driven assertions without duplicating
//! plumbing in each surface test bin.
//!
//! ## ADR alignment
//!
//! - **ADR-002 Client-Server Authority**: the helper is a read-only test
//!   primitive over post-layout bounding rectangles. No optimistic
//!   client-side authority is introduced.
//! - **ADR-021 Presentation Layer Architecture**: the helper does not
//!   reorder the `PresentationPlugin` composition. Overlay / modal layers
//!   are excluded from the geometric no-overlap check per story 002's
//!   named [`bevy::ui::GlobalZIndex`] hierarchy — story 002 already
//!   guarantees z-ordering at paint time.
//!
//! ## Friend-game scope preserved
//!
//! `QA-COND-0005` (Standard-tier accessibility ≥44px hit-target),
//! `QA-COND-0006` (playtest / fun-hypothesis validation), and
//! `PAW-TD-*-a` (placeholder-art accept-risk) are NOT advanced by this
//! helper. The invariants assert geometry only.

#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::BTreeMap;

/// One canonical viewport size from the Sprint 14 matrix.
///
/// Six entries cover 16:9 minimum / HD / 4K, 16:10, 4:3, and 21:9 ultrawide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ViewportSize {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
}

/// The six canonical viewport sizes asserted by every invariant suite run
/// (story 005 In Scope §1 + QA plan §"Tier 0 rank 4" AC1).
pub const CANONICAL_VIEWPORTS: [ViewportSize; 6] = [
    ViewportSize {
        name: "1366x768",
        width: 1366,
        height: 768,
    },
    ViewportSize {
        name: "1920x1080",
        width: 1920,
        height: 1080,
    },
    ViewportSize {
        name: "1920x1200",
        width: 1920,
        height: 1200,
    },
    ViewportSize {
        name: "1280x960",
        width: 1280,
        height: 960,
    },
    ViewportSize {
        name: "3840x2160",
        width: 3840,
        height: 2160,
    },
    ViewportSize {
        name: "2560x1080",
        width: 2560,
        height: 1080,
    },
];

/// Resource carrying the active viewport size into the Bevy test World so
/// systems / queries can reason about the synthesized framebuffer.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ActiveViewport(pub ViewportSize);

/// Z-layer classification used by the no-overlap rule. Per story 005 In
/// Scope §"No overlap" overlay / modal surfaces are detected by named
/// z-layer (story 002 design tokens), not by geometry — so they are
/// excluded from the pairwise overlap check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZLayer {
    /// Story 002 `UI_BASE` — foundational bevy_ui roots subject to the
    /// pairwise geometric no-overlap rule.
    UiBase,
    /// Story 002 `UI_OVERLAY` — translucent overlays painted above
    /// `UI_BASE`. Excluded from the geometric overlap check.
    UiOverlay,
    /// Story 002 `MODAL` — centred modal panels. Excluded from the
    /// geometric overlap check.
    Modal,
}

/// Surface kind — full UI root or deterministic-height strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceKind {
    /// A top-level UI root (lobby, draft panel, shop panel, etc.).
    Surface,
    /// A strip primitive (story 004 `HeaderBar` / `HandBar` / `FooterBar`)
    /// whose pixel height must be identical across all six viewports.
    Strip,
}

/// Display-phase tag scoping the pairwise no-overlap check. Surfaces with
/// different `DisplayPhase` values are mutually exclusive in time and
/// cannot co-occupy the screen, so they are NOT paired by the overlap
/// rule.
///
/// The matrix is deliberately conservative — InSessionBase surfaces (HUD
/// strips, hand UI) are visible in every gameplay phase, so they are
/// paired with every other in-session surface. Lobby is paired only with
/// itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DisplayPhase {
    /// Pre-game lobby phase. Only the lobby root is visible.
    Lobby,
    /// Steady-state in-session gameplay. HUD strips + HandBar.
    InSessionBase,
    /// `DRAFT_INITIAL` — the draft centered modal is up.
    DraftInitial,
    /// `DRAFT_SHOP` — the shop panel is up.
    DraftShop,
    /// `DRAFT_AUCTION` — the auction panel is up.
    DraftAuction,
    /// `RESOLUTION_SETTLEMENT` — settlement overlay is up.
    Settlement,
    /// `GAME_OVER` — result screen is up.
    GameOver,
}

/// Proportional anchor of a surface within the viewport. `(0.0, 0.0)` is
/// the top-left of the viewport; `(0.5, 0.5)` is the center; `(1.0, 1.0)`
/// is the bottom-right. The anchor MUST be the same proportional value
/// across all six viewports — that is the anchor-stability invariant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProportionalAnchor {
    pub x: f32,
    pub y: f32,
}

impl ProportionalAnchor {
    pub const TOP_LEFT: ProportionalAnchor = ProportionalAnchor { x: 0.0, y: 0.0 };
    pub const CENTER: ProportionalAnchor = ProportionalAnchor { x: 0.5, y: 0.5 };
    pub const BOTTOM_LEFT: ProportionalAnchor = ProportionalAnchor { x: 0.0, y: 1.0 };
}

/// Post-layout bounding rectangle of one UI root at one viewport. `(x, y)`
/// is the top-left corner of the rectangle in pixel coordinates with `+y`
/// pointing down (bevy_ui screen-space convention).
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct UiRootBounds {
    pub name: &'static str,
    pub phase: DisplayPhase,
    pub kind: SurfaceKind,
    pub z_layer: ZLayer,
    pub anchor: ProportionalAnchor,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiRootBounds {
    /// Returns `true` if this rectangle's bounding box is fully contained
    /// inside the viewport rectangle `[0, 0, vw, vh]`.
    pub fn fits_within(&self, viewport: ViewportSize) -> bool {
        let vw = viewport.width as f32;
        let vh = viewport.height as f32;
        self.x >= 0.0
            && self.y >= 0.0
            && self.x + self.width <= vw + f32::EPSILON
            && self.y + self.height <= vh + f32::EPSILON
    }

    /// Returns `true` if this rectangle overlaps `other` (positive area
    /// intersection). Edge-touching rectangles do NOT overlap.
    pub fn overlaps(&self, other: &UiRootBounds) -> bool {
        let x_overlap = self.x < other.x + other.width && other.x < self.x + self.width;
        let y_overlap = self.y < other.y + other.height && other.y < self.y + self.height;
        x_overlap && y_overlap
    }
}

/// Per-viewport bounding rectangles for a single UI surface, keyed by
/// viewport name. Used by the baseline-driven test harness.
#[derive(Clone, Debug)]
pub struct SurfaceBaseline {
    pub name: &'static str,
    pub phase: DisplayPhase,
    pub kind: SurfaceKind,
    pub z_layer: ZLayer,
    pub anchor: ProportionalAnchor,
    /// `(x, y, width, height)` indexed by [`ViewportSize::name`].
    pub per_viewport: &'static [(&'static str, f32, f32, f32, f32)],
    /// Required only for [`SurfaceKind::Strip`] surfaces — the
    /// deterministic pixel height that MUST be identical across all
    /// viewports. `None` for [`SurfaceKind::Surface`].
    pub strip_height_px: Option<f32>,
}

impl SurfaceBaseline {
    /// Looks up the canonical bounding rectangle for this surface at the
    /// named viewport. Returns `None` if the viewport is not in the
    /// per-viewport table for this surface.
    pub fn rect_for(&self, viewport_name: &str) -> Option<(f32, f32, f32, f32)> {
        self.per_viewport
            .iter()
            .find_map(|(name, x, y, w, h)| (*name == viewport_name).then_some((*x, *y, *w, *h)))
    }

    /// Resolves this surface to a [`UiRootBounds`] for the given viewport.
    pub fn resolve(&self, viewport: ViewportSize) -> Option<UiRootBounds> {
        let (x, y, width, height) = self.rect_for(viewport.name)?;
        Some(UiRootBounds {
            name: self.name,
            phase: self.phase,
            kind: self.kind,
            z_layer: self.z_layer,
            anchor: self.anchor,
            x,
            y,
            width,
            height,
        })
    }
}

/// Full baseline fixture — the list of surfaces and their canonical
/// bounds across all six viewports. Stored as a `&'static [...]` so it
/// can be authored as a `const` Rust table without a serde / RON
/// runtime dependency (story 005 §"Likely Files Touched" calls the
/// fixture format `TBD by the worker`; Rust-source baseline chosen for
/// zero added dep + best diff-readability).
#[derive(Clone, Debug)]
pub struct ViewportBaseline {
    pub surfaces: &'static [SurfaceBaseline],
}

/// Spawns one [`UiRootBounds`] entity per `surfaces × viewports` cell
/// from `baseline` into `app.world_mut()`, plus an [`ActiveViewport`]
/// resource for the current viewport. Each entity is tagged with a
/// [`BaselineViewportTag`] so the extractor can filter by viewport.
///
/// Story 005 AC2: this is the canonical `spawn_with_viewport` helper
/// function. Tier 1 surface stories may wrap it with extra plugin
/// composition once they migrate to live-app extraction.
pub fn spawn_with_viewport(
    app: &mut App,
    viewport: ViewportSize,
    baseline: &ViewportBaseline,
) -> Vec<Entity> {
    app.world_mut().insert_resource(ActiveViewport(viewport));
    let mut spawned = Vec::new();
    for surface in baseline.surfaces {
        if let Some(bounds) = surface.resolve(viewport) {
            let id = app
                .world_mut()
                .spawn((bounds, BaselineViewportTag { viewport }))
                .id();
            spawned.push(id);
        }
    }
    spawned
}

/// Tag component pairing a spawned [`UiRootBounds`] with the viewport
/// instance it was resolved against.
#[derive(Component, Clone, Copy, Debug)]
pub struct BaselineViewportTag {
    pub viewport: ViewportSize,
}

/// Returns every [`UiRootBounds`] spawned for the given viewport from the
/// World. Story 005 AC2: this is the canonical `extract_root_bounds`
/// helper.
pub fn extract_root_bounds(app: &mut App, viewport: ViewportSize) -> Vec<UiRootBounds> {
    let mut out = Vec::new();
    let mut query = app
        .world_mut()
        .query::<(&UiRootBounds, &BaselineViewportTag)>();
    for (bounds, tag) in query.iter(app.world()) {
        if tag.viewport == viewport {
            out.push(*bounds);
        }
    }
    out
}

/// Returns every [`UiRootBounds`] spawned for the given viewport whose
/// [`DisplayPhase`] matches `phase`. Used by the pairwise no-overlap
/// check to scope pairings to surfaces that co-exist on screen.
pub fn extract_root_bounds_for_phase(
    app: &mut App,
    viewport: ViewportSize,
    phase: DisplayPhase,
) -> Vec<UiRootBounds> {
    extract_root_bounds(app, viewport)
        .into_iter()
        .filter(|b| b.phase == phase || phases_co_exist(b.phase, phase))
        .collect()
}

/// Returns `true` if a surface tagged with `a` is on screen at the same
/// time as a surface tagged with `b`. `InSessionBase` surfaces (HUD
/// strips, hand UI) are visible in every gameplay phase, so they are
/// paired with every other in-session surface.
fn phases_co_exist(a: DisplayPhase, b: DisplayPhase) -> bool {
    use DisplayPhase::*;
    if a == b {
        return true;
    }
    matches!(
        (a, b),
        (InSessionBase, DraftInitial)
            | (DraftInitial, InSessionBase)
            | (InSessionBase, DraftShop)
            | (DraftShop, InSessionBase)
            | (InSessionBase, DraftAuction)
            | (DraftAuction, InSessionBase)
            | (InSessionBase, Settlement)
            | (Settlement, InSessionBase)
    )
}

/// Story 005 invariant — no-overlap. Returns `Err(message)` if any pair
/// of `UI_BASE` z-layer surfaces with co-existing display phases have
/// rectangles that overlap. Overlay / modal z-layer surfaces are
/// excluded — story 002's named z-layer hierarchy guarantees they paint
/// above `UI_BASE` and intentional overlap is the design intent.
pub fn assert_no_overlap(
    bounds: &[UiRootBounds],
    viewport: ViewportSize,
) -> Result<(), AssertionFailure> {
    let geom = bounds
        .iter()
        .filter(|b| b.z_layer == ZLayer::UiBase)
        .collect::<Vec<_>>();
    for i in 0..geom.len() {
        for j in (i + 1)..geom.len() {
            let a = geom[i];
            let b = geom[j];
            if phases_co_exist(a.phase, b.phase) && a.overlaps(b) {
                return Err(AssertionFailure {
                    invariant: Invariant::NoOverlap,
                    viewport,
                    surface: a.name,
                    other_surface: Some(b.name),
                    detail: format!(
                        "geometry overlap at viewport {}: {} (phase {:?}, rect [{:.1},{:.1} {:.1}x{:.1}]) \
                         overlaps {} (phase {:?}, rect [{:.1},{:.1} {:.1}x{:.1}])",
                        viewport.name,
                        a.name,
                        a.phase,
                        a.x,
                        a.y,
                        a.width,
                        a.height,
                        b.name,
                        b.phase,
                        b.x,
                        b.y,
                        b.width,
                        b.height,
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Story 005 invariant — no-clipping. Returns `Err(message)` if any
/// surface's bounding rectangle extends beyond the viewport rectangle.
pub fn assert_no_clipping(
    bounds: &[UiRootBounds],
    viewport: ViewportSize,
) -> Result<(), AssertionFailure> {
    for b in bounds {
        if !b.fits_within(viewport) {
            let vw = viewport.width as f32;
            let vh = viewport.height as f32;
            let edge = if b.x < 0.0 {
                "left"
            } else if b.y < 0.0 {
                "top"
            } else if b.x + b.width > vw {
                "right"
            } else if b.y + b.height > vh {
                "bottom"
            } else {
                "unknown"
            };
            return Err(AssertionFailure {
                invariant: Invariant::NoClipping,
                viewport,
                surface: b.name,
                other_surface: None,
                detail: format!(
                    "surface {} clips the {} edge of viewport {} (rect [{:.1},{:.1} {:.1}x{:.1}], viewport {}x{})",
                    b.name, edge, viewport.name, b.x, b.y, b.width, b.height, viewport.width, viewport.height,
                ),
            });
        }
    }
    Ok(())
}

/// Story 005 invariant — anchor stability. For each surface, the
/// proportional anchor MUST be identical across all six viewports. The
/// `ProportionalAnchor` field is encoded at the `SurfaceBaseline` level
/// (not per-viewport), so the check is structural: it walks the baseline
/// and confirms the resolved (x, y) pixel position matches `anchor *
/// (viewport_width, viewport_height)` to within a 1px tolerance for
/// every viewport.
pub fn assert_anchor_stability(baseline: &ViewportBaseline) -> Result<(), AssertionFailure> {
    for surface in baseline.surfaces {
        // Strip primitives (HeaderBar / HandBar / FooterBar) are composed via
        // a strip column whose offsets compose by stacking, not by a single
        // proportional anchor. The strip-height invariant is the canonical
        // determinism check for strips; anchor stability is restricted to
        // full UI roots.
        if surface.kind == SurfaceKind::Strip {
            continue;
        }
        for viewport in CANONICAL_VIEWPORTS {
            let Some((x, y, width, height)) = surface.rect_for(viewport.name) else {
                continue;
            };
            let expected_x = surface.anchor.x * viewport.width as f32 - surface.anchor.x * width;
            let expected_y = surface.anchor.y * viewport.height as f32 - surface.anchor.y * height;
            let dx = (x - expected_x).abs();
            let dy = (y - expected_y).abs();
            if dx > 1.0 || dy > 1.0 {
                return Err(AssertionFailure {
                    invariant: Invariant::AnchorStability,
                    viewport,
                    surface: surface.name,
                    other_surface: None,
                    detail: format!(
                        "anchor drift at viewport {}: {} expected anchor proportional ({:.3}, {:.3}) \
                         -> pixel ({:.1}, {:.1}); baseline rect places it at ({:.1}, {:.1}); \
                         dx={:.2} dy={:.2} both must be <= 1.0",
                        viewport.name,
                        surface.name,
                        surface.anchor.x,
                        surface.anchor.y,
                        expected_x,
                        expected_y,
                        x,
                        y,
                        dx,
                        dy,
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Story 005 invariant — strip-height determinism. For every
/// [`SurfaceKind::Strip`] surface the pixel height MUST be identical
/// across all six viewports and MUST equal the surface's declared
/// `strip_height_px`.
pub fn assert_strip_height_determinism(
    baseline: &ViewportBaseline,
) -> Result<(), AssertionFailure> {
    for surface in baseline.surfaces {
        if surface.kind != SurfaceKind::Strip {
            continue;
        }
        let Some(expected) = surface.strip_height_px else {
            return Err(AssertionFailure {
                invariant: Invariant::StripHeight,
                viewport: CANONICAL_VIEWPORTS[0],
                surface: surface.name,
                other_surface: None,
                detail: format!(
                    "strip surface {} is missing strip_height_px; every Strip surface MUST declare \
                     its deterministic height",
                    surface.name,
                ),
            });
        };
        let mut heights: BTreeMap<&'static str, f32> = BTreeMap::new();
        for viewport in CANONICAL_VIEWPORTS {
            if let Some((_, _, _, h)) = surface.rect_for(viewport.name) {
                heights.insert(viewport.name, h);
            }
        }
        for (viewport_name, height) in &heights {
            if (height - expected).abs() > f32::EPSILON {
                let viewport = CANONICAL_VIEWPORTS
                    .iter()
                    .find(|v| &v.name == viewport_name)
                    .copied()
                    .unwrap_or(CANONICAL_VIEWPORTS[0]);
                return Err(AssertionFailure {
                    invariant: Invariant::StripHeight,
                    viewport,
                    surface: surface.name,
                    other_surface: None,
                    detail: format!(
                        "strip-height drift at viewport {}: {} expected deterministic height {:.1}px; \
                         baseline rect height is {:.1}px",
                        viewport_name, surface.name, expected, height,
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Story 005 AC2: composed `assert_invariants_against_baseline` helper.
/// Runs all four invariant classes against the baseline at the supplied
/// viewport and returns the first failure (or `Ok(())` if all pass).
pub fn assert_invariants_against_baseline(
    app: &mut App,
    viewport: ViewportSize,
    baseline: &ViewportBaseline,
) -> Result<(), AssertionFailure> {
    let bounds = extract_root_bounds(app, viewport);
    assert_no_overlap(&bounds, viewport)?;
    assert_no_clipping(&bounds, viewport)?;
    assert_anchor_stability(baseline)?;
    assert_strip_height_determinism(baseline)?;
    Ok(())
}

/// Named invariant class — surfaced in [`AssertionFailure`] so test
/// reporters can label each failure with the failing class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invariant {
    NoOverlap,
    NoClipping,
    AnchorStability,
    StripHeight,
}

/// Structured assertion failure — carries the invariant class, viewport,
/// surface name(s), and a human-readable detail string. Used by both the
/// positive invariant suite and the AC5/AC6/AC7 negative tests so the
/// test bin output is uniform.
#[derive(Clone, Debug)]
pub struct AssertionFailure {
    pub invariant: Invariant,
    pub viewport: ViewportSize,
    pub surface: &'static str,
    pub other_surface: Option<&'static str>,
    pub detail: String,
}

impl std::fmt::Display for AssertionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:?} @ {}] {}",
            self.invariant, self.viewport.name, self.detail
        )
    }
}
