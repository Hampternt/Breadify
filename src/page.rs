//! The display list: a page as positioned primitives, and nothing else.
//!
//! This is the only interface between deciding where things go and drawing
//! them. The layout measures type and settles every millimetre; a renderer —
//! the PDF writer today, the app's preview later — walks the list and draws.
//! Neither renderer can measure or lay anything out, so the paper and the
//! screen cannot disagree about a page.

use crate::geometry::{Mm, Point, Pt, Rect};
use crate::text::Style;

/// An ink colour. The printed page is black on white with a few greys and one
/// red rule; see the design tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Colour {
    pub const fn grey(level: u8) -> Self {
        Self {
            red: level,
            green: level,
            blue: level,
        }
    }

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// Text, block rules, badge fill, the department box.
pub const BLACK: Colour = Colour::grey(0x00);
/// The logo panel.
pub const NEAR_BLACK: Colour = Colour::rgb(0x14, 0x11, 0x16);
/// Route-total product names.
pub const INK_SOFT: Colour = Colour::grey(0x22);
/// Quiet marker text, tick-box outlines.
pub const INK_QUIET: Colour = Colour::grey(0x3d);
/// Crate glyph fill.
pub const CRATE_FILL: Colour = Colour::grey(0x4a);
/// Masthead labels, column subtotals.
pub const LABEL: Colour = Colour::grey(0x55);
/// The page note line, a group's address.
pub const NOTE: Colour = Colour::grey(0x6b);
/// Supplier names, the footer.
pub const FAINT: Colour = Colour::grey(0x8a);
/// Order identifiers — the quietest ink on the page.
pub const FAINTEST: Colour = Colour::grey(0x9c);
/// Sub-block rules.
pub const RULE_SUB: Colour = Colour::grey(0xB8);
/// The footer rule.
pub const RULE_FOOT: Colour = Colour::grey(0xC9);
/// Bread-line rules.
pub const RULE_LINE: Colour = Colour::grey(0xE2);
/// The zebra tint behind every second bread line.
pub const ZEBRA: Colour = Colour::grey(0xF1);
/// The legend strip's band.
pub const LEGEND_BAND: Colour = Colour::grey(0xF4);
/// The masthead rule, and nothing else. From the Matvare Expressen mark.
pub const BRAND_RED: Colour = Colour::rgb(0xFF, 0x4F, 0x46);
/// Paper.
pub const WHITE: Colour = Colour::grey(0xFF);

/// How a rectangle is inked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub weight: Pt,
    pub colour: Colour,
}

/// Artwork the page places but does not draw itself.
///
/// The layout decides where it goes and how big it is; each renderer draws it
/// however it can — the PDF writer embeds the vector original, the on-screen
/// preview does something cheaper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    /// The Matvare Expressen mark. White type beside a red bag, so it only
    /// ever sits on a dark panel.
    Wordmark,
}

/// One thing to draw. Positions are final; nothing here is measured again.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// A run of text, positioned at the start of its baseline.
    Text {
        baseline_start: Point,
        text: String,
        style: Style,
        colour: Colour,
    },
    /// A straight line — every rule on the page.
    Rule {
        from: Point,
        to: Point,
        weight: Pt,
        colour: Colour,
    },
    /// A piece of artwork, fitted to a rectangle.
    Artwork { rect: Rect, art: Art },
    /// A filled or outlined box: tick boxes, crate glyphs, badges, tints.
    Box {
        rect: Rect,
        fill: Option<Colour>,
        stroke: Option<Stroke>,
        /// Corner radius, `0.0` for square corners.
        radius: Mm,
    },
}

impl Primitive {
    /// The same primitive, moved `down` the sheet.
    pub fn moved(&self, down: Mm) -> Self {
        match self {
            Self::Text {
                baseline_start,
                text,
                style,
                colour,
            } => Self::Text {
                baseline_start: baseline_start.offset(0.0, down),
                text: text.clone(),
                style: *style,
                colour: *colour,
            },
            Self::Rule {
                from,
                to,
                weight,
                colour,
            } => Self::Rule {
                from: from.offset(0.0, down),
                to: to.offset(0.0, down),
                weight: *weight,
                colour: *colour,
            },
            Self::Artwork { rect, art } => Self::Artwork {
                rect: Rect {
                    y: rect.y + down,
                    ..*rect
                },
                art: *art,
            },
            Self::Box {
                rect,
                fill,
                stroke,
                radius,
            } => Self::Box {
                rect: Rect {
                    y: rect.y + down,
                    ..*rect
                },
                fill: *fill,
                stroke: *stroke,
                radius: *radius,
            },
        }
    }
}

/// One sheet of paper.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Page {
    pub primitives: Vec<Primitive>,
}

impl Page {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a run of text with its baseline starting at `baseline_start`.
    pub fn text(
        &mut self,
        baseline_start: Point,
        text: impl Into<String>,
        style: Style,
        colour: Colour,
    ) {
        self.primitives.push(Primitive::Text {
            baseline_start,
            text: text.into(),
            style,
            colour,
        });
    }

    /// Draws a horizontal rule of `width` starting at `from`.
    pub fn horizontal_rule(&mut self, from: Point, width: Mm, weight: Pt, colour: Colour) {
        self.primitives.push(Primitive::Rule {
            from,
            to: from.offset(width, 0.0),
            weight,
            colour,
        });
    }

    /// Draws a vertical rule of `height` starting at `from`.
    pub fn vertical_rule(&mut self, from: Point, height: Mm, weight: Pt, colour: Colour) {
        self.primitives.push(Primitive::Rule {
            from,
            to: from.offset(0.0, height),
            weight,
            colour,
        });
    }

    /// Fills a box.
    pub fn fill(&mut self, rect: Rect, colour: Colour) {
        self.primitives.push(Primitive::Box {
            rect,
            fill: Some(colour),
            stroke: None,
            radius: 0.0,
        });
    }

    /// Places a piece of artwork in a rectangle.
    pub fn artwork(&mut self, rect: Rect, art: Art) {
        self.primitives.push(Primitive::Artwork { rect, art });
    }

    /// Outlines a box.
    pub fn outline(&mut self, rect: Rect, weight: Pt, colour: Colour, radius: Mm) {
        self.primitives.push(Primitive::Box {
            rect,
            fill: None,
            stroke: Some(Stroke { weight, colour }),
            radius,
        });
    }

    /// A box that is both filled and outlined — the crate-of-five glyph, the
    /// no-substitutes badge.
    pub fn filled_outline(
        &mut self,
        rect: Rect,
        fill: Colour,
        weight: Pt,
        stroke: Colour,
        radius: Mm,
    ) {
        self.primitives.push(Primitive::Box {
            rect,
            fill: Some(fill),
            stroke: Some(Stroke {
                weight,
                colour: stroke,
            }),
            radius,
        });
    }

    /// Places everything from `other` on this page, moved `down` the sheet.
    ///
    /// This is how a block that was laid out on its own — at `y = 0`, so its
    /// height could be measured by laying it out rather than by a second
    /// formula that might disagree — reaches the sheet it belongs on.
    pub fn absorb(&mut self, other: &Page, down: Mm) {
        self.primitives.extend(
            other
                .primitives
                .iter()
                .map(|primitive| primitive.moved(down)),
        );
    }

    /// How far down the page the lowest thing drawn reaches. The pagination
    /// rules are stated against this.
    pub fn lowest_point(&self) -> Mm {
        self.primitives
            .iter()
            .map(|primitive| match primitive {
                Primitive::Text { baseline_start, .. } => baseline_start.y,
                Primitive::Rule { from, to, .. } => from.y.max(to.y),
                Primitive::Artwork { rect, .. } | Primitive::Box { rect, .. } => rect.bottom(),
            })
            .fold(0.0_f64, f64::max)
    }
}
