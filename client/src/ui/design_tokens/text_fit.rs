//! Text-fitting / wrapping-policy primitive — Sprint 17 UI layout
//! foundation (PROMPT 1181 / `S17-UI-LAYOUT-FOUNDATION-PRIMITIVES`).
//!
//! Three classes of text in the playable client want three different
//! line-break treatments:
//!
//! 1. **Single-line, never-wrap labels** — HUD gold readout, status
//!    chips, button labels, lobby room-code chip. Wrapping would split
//!    a glyph block in half mid-readout (visible regression).
//! 2. **Wrap-to-width body copy** — modal-panel body text, lobby
//!    welcome strings, photosensitivity warning paragraphs. These
//!    must wrap at word boundaries so long strings stay legible.
//! 3. **Single-line with overflow clip** — surface titles that should
//!    truncate visually if their parent shrinks unexpectedly rather
//!    than wrap into a second line that would push the body or CTA
//!    off-screen.
//!
//! Today every surface chooses its own `TextLayout` and `LineBreak` at
//! the spawn site, often with no explicit policy. This module names
//! the three canonical policies and exports a `TextLayout` factory
//! per policy so a surface declares intent rather than re-deriving
//! the bevy enum at the call site.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: text-fit policies
//!   are read-only presentation primitives.
//! - **ADR-002 Client-Server Authority**: no game state.

use bevy::text::{Justify, LineBreak, TextLayout};

/// Canonical text-fit policies. Each variant binds a [`LineBreak`] mode
/// + a default [`Justify`] alignment so the spawn site does not have to
/// re-derive both. Surfaces MAY override the alignment via the
/// `with_justify_*` constructors below.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextFitPolicy {
    /// Single-line, never-wrap. `LineBreak::NoWrap` + `Justify::Left`.
    /// Canonical use: status chip, HUD readout, button label,
    /// room-code chip.
    SingleLineNoWrap,
    /// Wrap at word boundaries. `LineBreak::WordBoundary` +
    /// `Justify::Left`. Canonical use: modal-panel body copy, lobby
    /// welcome strings, photosensitivity warning paragraphs.
    WrapWordBoundary,
    /// Wrap at character boundaries when a word cannot fit.
    /// `LineBreak::WordOrCharacter` + `Justify::Left`. Canonical use:
    /// localized strings that may produce single tokens longer than
    /// the panel width.
    WrapWordOrCharacter,
}

impl TextFitPolicy {
    /// The bevy `LineBreak` mode bound to this policy.
    pub const fn line_break(self) -> LineBreak {
        match self {
            Self::SingleLineNoWrap => LineBreak::NoWrap,
            Self::WrapWordBoundary => LineBreak::WordBoundary,
            Self::WrapWordOrCharacter => LineBreak::WordOrCharacter,
        }
    }

    /// `true` iff this policy will produce more than one line under any
    /// non-degenerate text. The text-fit invariant tests use this to
    /// assert that "fit one line" surfaces never pick a wrap policy.
    pub const fn allows_soft_wrap(self) -> bool {
        !matches!(self, Self::SingleLineNoWrap)
    }
}

/// Build a [`TextLayout`] for the given policy with default
/// `Justify::Left` alignment.
pub fn text_layout(policy: TextFitPolicy) -> TextLayout {
    TextLayout::new(Justify::Left, policy.line_break())
}

/// Build a [`TextLayout`] with custom alignment.
pub fn text_layout_with_justify(policy: TextFitPolicy, justify: Justify) -> TextLayout {
    TextLayout::new(justify, policy.line_break())
}

/// Convenience constructor for a center-justified single-line label
/// (CTA buttons, status chips usually want this).
pub fn single_line_centered() -> TextLayout {
    text_layout_with_justify(TextFitPolicy::SingleLineNoWrap, Justify::Center)
}

/// Convenience constructor for a left-justified wrap-to-width body.
pub fn wrap_body_left() -> TextLayout {
    text_layout(TextFitPolicy::WrapWordBoundary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_maps_to_canonical_line_break_modes() {
        assert_eq!(
            TextFitPolicy::SingleLineNoWrap.line_break(),
            LineBreak::NoWrap
        );
        assert_eq!(
            TextFitPolicy::WrapWordBoundary.line_break(),
            LineBreak::WordBoundary
        );
        assert_eq!(
            TextFitPolicy::WrapWordOrCharacter.line_break(),
            LineBreak::WordOrCharacter
        );
    }

    #[test]
    fn single_line_policy_does_not_allow_soft_wrap() {
        assert!(!TextFitPolicy::SingleLineNoWrap.allows_soft_wrap());
        assert!(TextFitPolicy::WrapWordBoundary.allows_soft_wrap());
        assert!(TextFitPolicy::WrapWordOrCharacter.allows_soft_wrap());
    }

    #[test]
    fn text_layout_pairs_line_break_with_supplied_justify() {
        let layout = text_layout_with_justify(TextFitPolicy::WrapWordBoundary, Justify::Right);
        assert_eq!(layout.linebreak, LineBreak::WordBoundary);
        assert_eq!(layout.justify, Justify::Right);
    }

    #[test]
    fn single_line_centered_helper_is_no_wrap_center() {
        let layout = single_line_centered();
        assert_eq!(layout.linebreak, LineBreak::NoWrap);
        assert_eq!(layout.justify, Justify::Center);
    }

    #[test]
    fn wrap_body_left_helper_is_word_boundary_left() {
        let layout = wrap_body_left();
        assert_eq!(layout.linebreak, LineBreak::WordBoundary);
        assert_eq!(layout.justify, Justify::Left);
    }

    #[test]
    fn no_wrap_policy_must_never_match_wrap_modes() {
        // Drift-guard: a future refactor that aliases SingleLineNoWrap
        // onto WordBoundary would silently introduce regression
        // wrapping on every HUD readout and status chip.
        let no_wrap = TextFitPolicy::SingleLineNoWrap.line_break();
        let word_boundary = TextFitPolicy::WrapWordBoundary.line_break();
        let word_or_char = TextFitPolicy::WrapWordOrCharacter.line_break();
        assert_ne!(no_wrap, word_boundary);
        assert_ne!(no_wrap, word_or_char);
        assert_ne!(word_boundary, word_or_char);
    }
}
