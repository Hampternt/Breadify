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
    CONTENT_WIDTH, FOOTER_CLEARANCE, MARGIN_BOTTOM, MARGIN_SIDE, Mm, PAGE_HEIGHT, Point, Pt, Rect,
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

/// One printed sheet: which route it belongs to, where it falls in that
/// route's set, and everything on it.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub route: String,
    /// 1-based within the route.
    pub number: usize,
    pub of: usize,
    pub content: Page,
}

/// A block waiting for a page, already laid out and therefore already
/// measured.
struct Piece {
    content: Page,
    height: Mm,
    /// A separator that must never be the last thing on a page: the
    /// unsequenced flag belongs above the stops it covers.
    keep_with_next: bool,
}

/// Every sheet a day's routes need, in printing order.
///
/// Routes come in the order they are given, each starting on a fresh page. No
/// page ever carries two routes (decision D1).
pub fn day(
    routes: &[Route],
    dates: Option<DeliveryDates>,
    rules: &CrateRules,
    source: &str,
) -> Vec<Sheet> {
    routes
        .iter()
        .flat_map(|route| paginate(route, dates, rules, source))
        .collect()
}

/// Lays one route out over as many sheets as it needs.
pub fn paginate(
    route: &Route,
    dates: Option<DeliveryDates>,
    rules: &CrateRules,
    source: &str,
) -> Vec<Sheet> {
    let column = Cursor::new(0.0);
    let pieces = pieces(route, rules, &column);
    let breaks = share_out(&pieces, content_top(route), content_limit());

    let count = breaks.len().max(1);
    breaks
        .into_iter()
        .enumerate()
        .map(|(index, page_pieces)| {
            let context = SheetContext {
                dates,
                page: index + 1,
                pages: count,
                route_stops: route.stops.len(),
                route_lines: route.line_count(),
                source: source.to_owned(),
            };
            Sheet {
                route: route.nickname.clone(),
                number: index + 1,
                of: count,
                content: compose(route, &context, &pieces, &page_pieces),
            }
        })
        .collect()
}

/// Everything a route puts on paper, in order: its stops, the flag above the
/// unsequenced ones, and the total that closes it.
fn pieces(route: &Route, rules: &CrateRules, column: &Cursor) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut flagged = false;

    for entry in &route.stops {
        if !entry.is_sequenced() && !flagged {
            flagged = true;
            let (content, height) = furniture::unsequenced_flag_block(column);
            pieces.push(Piece {
                content,
                height,
                keep_with_next: true,
            });
        }
        let (content, height) = stop::block(entry, rules, column);
        pieces.push(Piece {
            content,
            height,
            keep_with_next: false,
        });
    }

    let (content, height) = total::block(route, column);
    pieces.push(Piece {
        content,
        height,
        keep_with_next: false,
    });
    pieces
}

/// Decides which pieces go on which page.
///
/// A piece never splits, and a piece marked `keep_with_next` never ends a
/// page. When the last page would carry nothing but the route total, the stop
/// above it comes down too rather than leave a sheet nearly empty.
fn share_out(pieces: &[Piece], top: Mm, limit: Mm) -> Vec<Vec<usize>> {
    let mut pages: Vec<Vec<usize>> = vec![Vec::new()];
    let mut y = top;

    for (index, piece) in pieces.iter().enumerate() {
        let needed = piece.height + follower(pieces, index);
        let fits = y + needed <= limit;

        if !fits && !pages.last().is_some_and(Vec::is_empty) {
            pages.push(Vec::new());
            y = top;
        }

        pages.last_mut().expect("a page to place onto").push(index);
        y += piece.height;
    }

    rebalance(pieces, &mut pages, top, limit);
    pages
}

/// How much room the piece after this one needs, when the two must stay
/// together.
fn follower(pieces: &[Piece], index: usize) -> Mm {
    if !pieces[index].keep_with_next {
        return 0.0;
    }
    pieces.get(index + 1).map_or(0.0, |piece| piece.height)
}

/// Pulls the previous stop down onto a final page that carries only the total.
fn rebalance(pieces: &[Piece], pages: &mut [Vec<usize>], top: Mm, limit: Mm) {
    if pages.len() < 2 {
        return;
    }
    let Some(last) = pages.last() else {
        return;
    };
    if last.len() != 1 {
        return;
    }

    let total = last[0];
    let previous = pages.len() - 2;
    let Some(&moved) = pages[previous].last() else {
        return;
    };
    if pieces[moved].keep_with_next || pages[previous].len() < 2 {
        return;
    }

    // Moving this one down must not leave a separator as the last thing on the
    // page it came from — the flag belongs above the stops it covers.
    let left_behind = pages[previous][pages[previous].len() - 2];
    if pieces[left_behind].keep_with_next {
        return;
    }

    let height = pieces[moved].height + pieces[total].height;
    if top + height > limit {
        return;
    }

    pages[previous].pop();
    pages.last_mut().expect("the final page").insert(0, moved);
}

/// Where a page's own content starts, below the furniture every page repeats.
fn content_top(route: &Route) -> Mm {
    let mut scratch = Page::new();
    let mut cursor = Cursor::new(0.0);
    let context = SheetContext {
        dates: None,
        page: 1,
        pages: 1,
        route_stops: route.stops.len(),
        route_lines: route.line_count(),
        source: String::new(),
    };
    furniture::masthead(&mut scratch, &mut cursor, route, &context);
    furniture::page_note(&mut scratch, &mut cursor, route);
    furniture::legend(&mut scratch, &mut cursor);
    cursor.y
}

/// How far down a page content may reach: the footer's rule, less the
/// clearance every page keeps above it.
fn content_limit() -> Mm {
    footer_rule_y() - FOOTER_CLEARANCE
}

/// Draws one page: the furniture it repeats, then its share of the pieces.
fn compose(route: &Route, context: &SheetContext, pieces: &[Piece], on_page: &[usize]) -> Page {
    let mut page = Page::new();
    let mut cursor = Cursor::new(0.0);

    furniture::masthead(&mut page, &mut cursor, route, context);
    furniture::page_note(&mut page, &mut cursor, route);
    furniture::legend(&mut page, &mut cursor);

    for &index in on_page {
        page.absorb(&pieces[index].content, cursor.y);
        cursor.advance(pieces[index].height);
    }

    furniture::footer(&mut page, route, context);
    page
}

/// Lays a whole route out on one sheet, for a route that fits.
///
/// # Panics
///
/// If the route needs more than one sheet — use [`paginate`] for those.
pub fn sheet(route: &Route, context: &SheetContext, rules: &CrateRules) -> Page {
    let sheets = paginate(route, context.dates, rules, &context.source);
    assert_eq!(
        sheets.len(),
        1,
        "route {} needs {} sheets — use paginate",
        route.nickname,
        sheets.len()
    );
    sheets.into_iter().next().expect("one sheet").content
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
