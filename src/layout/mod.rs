//! Laying a route out on a sheet.
//!
//! Turns a route into a [`Page`] of positioned primitives. Everything here
//! decides *where*; nothing draws. Measurements come from [`crate::text`], so
//! the same list can be drawn to paper or to a screen without either one
//! laying it out again.

pub mod furniture;
pub mod metrics;
pub mod stop;
pub mod total;

use crate::crates::CrateRules;
use crate::date::DeliveryDates;
use crate::font::Face;
use crate::geometry::{
    CONTENT_WIDTH, MARGIN_BOTTOM, MARGIN_SIDE, Mm, PAGE_HEIGHT, Point, Pt, Rect,
};
use crate::page::{self, Colour, Page};
use crate::route::Route;
use crate::text::{self, Style};

/// Where the next thing goes, as a sheet fills from the top.
pub struct Cursor {
    /// Distance from the top of the sheet to the next free millimetre.
    pub y: Mm,
    /// The left edge of the content column.
    pub left: Mm,
    /// The content column's width.
    pub width: Mm,
}

impl Cursor {
    pub fn new(y: Mm) -> Self {
        Self {
            y,
            left: MARGIN_SIDE,
            width: CONTENT_WIDTH,
        }
    }

    pub fn right(&self) -> Mm {
        self.left + self.width
    }

    /// Moves down the page.
    pub fn advance(&mut self, by: Mm) {
        self.y += by;
    }
}

/// What a sheet says about its place in the day's printing.
#[derive(Debug, Clone, PartialEq)]
pub struct SheetContext {
    pub dates: Option<DeliveryDates>,
    /// 1-based, for `Page 1 of 2`.
    pub page: usize,
    pub pages: usize,
    /// Stops and lines on the whole route, for the masthead counter.
    pub route_stops: usize,
    pub route_lines: usize,
    /// The export's filename, for the footer.
    pub source: String,
}

impl SheetContext {
    /// A single-page route, which is what pack 2 draws.
    pub fn single(route: &Route, dates: Option<DeliveryDates>, source: impl Into<String>) -> Self {
        Self {
            dates,
            page: 1,
            pages: 1,
            route_stops: route.stops.len(),
            route_lines: route.line_count(),
            source: source.into(),
        }
    }

    pub fn is_continuation(&self) -> bool {
        self.page > 1
    }

    pub fn continues(&self) -> bool {
        self.page < self.pages
    }
}

/// Lays out a whole route on one sheet.
///
/// Pack 2 draws routes that fit; the paginator that splits the ones that do
/// not is pack 3, and it will reuse every piece of this.
pub fn sheet(route: &Route, context: &SheetContext, rules: &CrateRules) -> Page {
    let mut page = Page::new();
    let mut cursor = Cursor::new(0.0);

    furniture::masthead(&mut page, &mut cursor, route, context);
    furniture::page_note(&mut page, &mut cursor, route);
    furniture::legend(&mut page, &mut cursor);

    let mut flagged = false;
    for entry in &route.stops {
        if !entry.is_sequenced() && !flagged {
            flagged = true;
            furniture::unsequenced_flag(&mut page, &mut cursor);
        }
        stop::block(&mut page, &mut cursor, entry, rules);
    }

    total::block(&mut page, &mut cursor, route);
    furniture::footer(&mut page, route, context);
    page
}

/// Sets a run of text so its left edge sits at `at`, with `at.y` the top of
/// the line rather than its baseline — which is how the layout thinks.
pub fn text_from_top(page: &mut Page, at: Point, run: &str, style: Style, colour: Colour) -> Mm {
    let baseline = at.offset(0.0, text::ascent(style));
    page.text(baseline, run, style, colour);
    text::line_height(style)
}

/// The same, right-aligned so the run *ends* at `right`.
pub fn text_from_right(
    page: &mut Page,
    right: Mm,
    top: Mm,
    run: &str,
    style: Style,
    colour: Colour,
) -> Mm {
    let width = text::width(run, style);
    text_from_top(page, Point::new(right - width, top), run, style, colour)
}

/// A horizontal rule across the content column.
pub fn rule(page: &mut Page, cursor: &Cursor, weight: Pt, colour: Colour) {
    page.horizontal_rule(
        Point::new(cursor.left, cursor.y),
        cursor.width,
        weight,
        colour,
    );
}

/// The mono face at a size, tracked the way micro-labels are.
pub fn micro_label(size: Pt) -> Style {
    Style::new(Face::MonoRegular, size).tracked(metrics::TRACK_MICRO_LABEL)
}

/// Where the footer's rule sits: hard at the bottom of the sheet.
pub fn footer_rule_y() -> Mm {
    PAGE_HEIGHT
        - MARGIN_BOTTOM
        - text::line_height(Style::new(Face::MonoRegular, metrics::SIZE_FOOTER))
        - metrics::FOOTER_PADDING_TOP
}

/// Draws one crate glyph. A crate of ten is filled; a crate of five is filled
/// only in its lower half, so ink level reads as capacity.
pub fn crate_glyph(page: &mut Page, at: Point, full: bool) {
    let (width, height) = metrics::CRATE_GLYPH;
    let rect = Rect::new(at.x, at.y, width, height);

    if full {
        page.filled_outline(
            rect,
            page::CRATE_FILL,
            metrics::RULE_BOX,
            page::BLACK,
            metrics::CRATE_GLYPH_RADIUS,
        );
        return;
    }

    page.fill(
        Rect::new(at.x, at.y + height / 2.0, width, height / 2.0),
        page::CRATE_FILL,
    );
    page.outline(
        rect,
        metrics::RULE_BOX,
        page::BLACK,
        metrics::CRATE_GLYPH_RADIUS,
    );
}
