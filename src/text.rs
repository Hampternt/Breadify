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
///
/// Tracking counts the gaps *between* glyphs, which is the inked extent. PDF's
/// own character spacing also advances the cursor past the last glyph, so a
/// tracked run leaves the cursor a fraction further right than this reports —
/// under 0.25 mm at these sizes, and nothing positions anything from where a
/// run ends.
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

/// Breaks `text` into lines that each fit `width`, at its spaces.
///
/// A word too long to fit on a line of its own is broken between characters
/// rather than left to run off the page — a product name is one long word more
/// often than it looks (`Fiskegrateng`), and the alternative is a line the
/// printer silently loses.
///
/// An empty `text`, or a `width` no character fits in, gives one line back, so
/// a caller can always count on at least one.
pub fn wrap(text: &str, style: Style, width: Mm) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if self::width(&candidate, style) <= width {
            line = candidate;
            continue;
        }

        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        // The word alone, which may still be too long for the line.
        for piece in break_word(word, style, width) {
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            line = piece;
        }
    }

    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// Splits one over-long word between characters, into pieces that each fit.
fn break_word(word: &str, style: Style, width: Mm) -> Vec<String> {
    if self::width(word, style) <= width {
        return vec![word.to_owned()];
    }

    let mut pieces: Vec<String> = Vec::new();
    let mut piece = String::new();
    for character in word.chars() {
        let mut candidate = piece.clone();
        candidate.push(character);
        if !piece.is_empty() && self::width(&candidate, style) > width {
            pieces.push(std::mem::take(&mut piece));
        }
        piece.push(character);
    }
    if !piece.is_empty() {
        pieces.push(piece);
    }
    pieces
}
