//! Measuring type, headlessly.
//!
//! Widths come straight out of the font's own tables, never from a UI
//! toolkit's layout: a toolkit rounds and pixel-snaps its geometry by design,
//! which would put the preview and the paper a fraction of a millimetre apart
//! on every run. The paginator asks this module how wide a run is, and both
//! renderers only draw what it decided.

use crate::font::Face;
use crate::geometry::{Mm, Pt, pt_to_mm};

/// How a run of text is set: which face, how big, and how tightly tracked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub face: Face,
    pub size: Pt,
    /// Letter-spacing as a fraction of the type size, the way the design
    /// states it: `-0.02` tightens, `0.08` opens up.
    pub tracking_em: f64,
}

impl Style {
    /// A run with the design's default tracking for its face — none.
    pub fn new(face: Face, size: Pt) -> Self {
        Self {
            face,
            size,
            tracking_em: 0.0,
        }
    }

    /// The same style, tracked.
    pub fn tracked(self, tracking_em: f64) -> Self {
        Self {
            tracking_em,
            ..self
        }
    }
}

/// How wide `text` is when set in `style`.
///
/// Sums the glyphs' own advances and the tracking between them. It does not
/// apply kerning pairs, which for these faces and this Latin text only ever
/// tighten a line — so a run that fits by this measure fits when printed.
pub fn width(text: &str, style: Style) -> Mm {
    if text.is_empty() {
        return 0.0;
    }

    let face = style.face.parsed();
    let units_per_em = f64::from(face.units_per_em());

    let mut advances: f64 = 0.0;
    let mut glyphs: f64 = 0.0;
    for character in text.chars() {
        glyphs += 1.0;
        let Some(glyph) = face.glyph_index(character) else {
            continue;
        };
        advances += f64::from(face.glyph_hor_advance(glyph).unwrap_or(0));
    }

    let em_width = advances / units_per_em;
    let tracking = style.tracking_em * (glyphs - 1.0).max(0.0);
    pt_to_mm((em_width + tracking) * style.size)
}

/// How tall a line of `style` is: the face's own ascent plus descent, which is
/// what a block of them stacks by.
pub fn line_height(style: Style) -> Mm {
    let face = style.face.parsed();
    let units_per_em = f64::from(face.units_per_em());
    let height = f64::from(face.ascender() - face.descender());
    pt_to_mm(height / units_per_em * style.size)
}

/// How far the tallest letters reach above the baseline.
pub fn ascent(style: Style) -> Mm {
    let face = style.face.parsed();
    let units_per_em = f64::from(face.units_per_em());
    pt_to_mm(f64::from(face.ascender()) / units_per_em * style.size)
}
