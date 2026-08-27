//! The page's furniture: masthead, page note, legend strip, the unsequenced
//! flag and the footer.

use super::metrics::*;
use super::{Cursor, micro_label, rule, text_from_right, text_from_top};
use crate::font::Face;
use crate::geometry::{MARGIN_SIDE, MARGIN_TOP, Mm, PAGE_WIDTH, Point, Rect};
use crate::layout::{Settings, SheetContext};
use crate::page::{self, Page};
use crate::route::Route;
use crate::text::{self, Style};

/// The logo panel, the route, the date and the page counter, over the brand
/// rule.
pub fn masthead(
    page: &mut Page,
    cursor: &mut Cursor,
    route: &Route,
    context: &SheetContext,
    settings: &Settings,
) {
    cursor.y = MARGIN_TOP;
    let top = cursor.y;

    logo_panel(page, Point::new(cursor.left, top));

    let label_left = cursor.left + LOGO_PANEL.0 + 2.0 * LOGO_PADDING.1 + HEADING_GAP;
    let label = Style::new(Face::MonoRegular, SIZE_ROUTE_LABEL).tracked(TRACK_MICRO_LABEL);
    // A driver holding both days' sheets should not have to read the footer's
    // filename to tell them apart. Bread says nothing, as it always has.
    let heading = if settings.list.names_itself() {
        format!("{} ROUTE", settings.list.word())
    } else {
        "ROUTE".to_owned()
    };
    text_from_top(
        page,
        Point::new(label_left, top),
        &heading,
        label,
        page::LABEL,
    );

    let number = Style::new(Face::ArchivoBlack, SIZE_ROUTE_NUMBER).tracked(TRACK_ROUTE_NUMBER);
    let number_top = top + text::line_height(label) * 0.6;
    let nickname = route.nickname.to_uppercase();
    text_from_top(
        page,
        Point::new(label_left, number_top),
        &nickname,
        number,
        page::BLACK,
    );

    if context.is_continuation() {
        let continued = Style::new(Face::MonoRegular, SIZE_CONTINUED).tracked(TRACK_CONTINUED);
        let after = label_left + text::width(&nickname, number) + HEADING_GAP;
        text_from_top(
            page,
            Point::new(
                after,
                number_top + text::ascent(number) - text::ascent(continued),
            ),
            "CONTINUED",
            continued,
            page::LABEL,
        );
    }

    let date = context
        .dates
        .map_or_else(|| "date unknown".to_owned(), |dates| dates.to_string());
    let date_style = Style::new(Face::MonoMedium, SIZE_BADGE);
    text_from_right(page, cursor.right(), top, &date, date_style, page::BLACK);

    let counter = format!(
        "PAGE {} OF {} · {} STOPS · {} LINES",
        context.page, context.pages, context.route_stops, context.route_lines
    );
    text_from_right(
        page,
        cursor.right(),
        top + text::line_height(date_style) + 0.6,
        &counter,
        micro_label(SIZE_PAGE_COUNTER).tracked(TRACK_TAG),
        page::LABEL,
    );

    let block_bottom = number_top + text::line_height(number);
    cursor.y = block_bottom + MASTHEAD_RULE_GAP;
    rule(page, cursor, RULE_MASTHEAD, page::BRAND_RED);
    cursor.advance(2.6);
}

/// The wordmark on its dark panel. The mark is white type beside a red bag, so
/// it can only sit on the panel — never on the paper.
fn logo_panel(page: &mut Page, at: Point) {
    let (width, height) = LOGO_PANEL;
    let panel = Rect::new(
        at.x,
        at.y,
        width + 2.0 * LOGO_PADDING.1,
        height + 2.0 * LOGO_PADDING.0,
    );
    page.fill(panel, page::NEAR_BLACK);
    page.artwork(
        Rect::new(at.x + LOGO_PADDING.1, at.y + LOGO_PADDING.0, width, height),
        page::Art::Wordmark,
    );
}

/// One sentence of context on the left, the substitute convention on the
/// right.
pub fn page_note(page: &mut Page, cursor: &mut Cursor, route: &Route) {
    let style = Style::new(Face::MonoRegular, SIZE_PAGE_NOTE);
    let unsequenced = route.unsequenced().count();

    let left = match unsequenced {
        0 => format!(
            "Route {} in full — {} stops.",
            route.nickname,
            route.stops.len()
        ),
        _ => format!(
            "Route {} in full — {} stops, {unsequenced} with no position assigned.",
            route.nickname,
            route.stops.len()
        ),
    };
    let height = text_from_top(
        page,
        Point::new(cursor.left, cursor.y),
        &left,
        style,
        page::NOTE,
    );
    text_from_right(
        page,
        cursor.right(),
        cursor.y,
        "want substitute: true unless marked FALSE",
        style,
        page::NOTE,
    );

    cursor.advance(height + 1.4);
}

/// The tinted band that explains the boxes, the crate glyphs and the supplier
/// codes.
pub fn legend(page: &mut Page, cursor: &mut Cursor, route: &Route, settings: &Settings) {
    let text_style = Style::new(Face::MonoRegular, SIZE_LEGEND);
    let bold = Style::new(Face::MonoBold, SIZE_LEGEND).tracked(TRACK_TAG);
    let height = SWATCH + 2.0 * LEGEND_PADDING.0;
    let band = Rect::new(cursor.left, cursor.y, cursor.width, height);

    page.fill(band, page::LEGEND_BAND);
    page.horizontal_rule(
        Point::new(band.x, band.y),
        band.width,
        RULE_FOOTER,
        page::RULE_FOOT,
    );
    page.horizontal_rule(
        Point::new(band.x, band.bottom()),
        band.width,
        RULE_FOOTER,
        page::RULE_FOOT,
    );

    let middle = band.y + height / 2.0;
    let mut x = band.x + LEGEND_PADDING.1;

    x += write_middle(page, x, middle, "BOXES", bold, page::BLACK) + LEGEND_ITEM_GAP;
    for (letter, word) in [("P", "Picked"), ("M", "Missing"), ("F", "Fixed")] {
        swatch(page, Point::new(x, middle - SWATCH / 2.0), letter);
        x += SWATCH + 1.0;
        x += write_middle(page, x, middle, word, text_style, page::INK_QUIET) + LEGEND_ITEM_GAP;
    }

    if settings.has_crates() {
        x += write_middle(page, x, middle, "CRATES", bold, page::BLACK) + LEGEND_ITEM_GAP;
        for (full, count) in [(true, "10"), (false, "5")] {
            super::crate_glyph(page, Point::new(x, middle - CRATE_GLYPH.1 / 2.0), full);
            x += CRATE_GLYPH.0 + 1.0;
            x +=
                write_middle(page, x, middle, count, text_style, page::INK_QUIET) + LEGEND_ITEM_GAP;
        }
    }

    supplier_key(page, band, middle, x, route, text_style);
    cursor.y = band.bottom() + 2.2;
}

/// What the two-letter code on every line stands for, right-aligned in the
/// legend band.
///
/// It used to name the two bakeries whatever the sheet was for. The freezer
/// list has seven suppliers on one route and none of them a bakery, so the key
/// comes from the route — spelled out if the room left in the band takes it,
/// codes alone if not, and nothing at all when even those will not fit. The
/// route total spells every supplier out regardless, so nothing is lost.
fn supplier_key(page: &mut Page, band: Rect, middle: Mm, used: Mm, route: &Route, style: Style) {
    let mut names: Vec<&str> = route
        .stops
        .iter()
        .flat_map(|stop| stop.lines.iter())
        .map(|line| line.product.supplier.as_str())
        .collect();
    names.sort_by_key(|name| crate::supplier::column_position(name));
    names.dedup();
    if names.is_empty() {
        return;
    }

    let room = band.right() - LEGEND_PADDING.1 - used;
    let spelled = names
        .iter()
        .map(|name| {
            format!(
                "{} {}",
                crate::supplier::code(name),
                crate::supplier::display_name(name)
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let codes = names
        .iter()
        .map(|name| crate::supplier::code(name))
        .collect::<Vec<_>>()
        .join(" · ");

    let key = if text::width(&spelled, style) <= room {
        spelled
    } else if text::width(&codes, style) <= room {
        codes
    } else {
        return;
    };

    let width = text::width(&key, style);
    write_middle(
        page,
        band.right() - LEGEND_PADDING.1 - width,
        middle,
        &key,
        style,
        page::INK_QUIET,
    );
}

/// A legend swatch: the tick box with its letter, at a quarter strength so it
/// reads as an empty box rather than a ticked one.
fn swatch(page: &mut Page, at: Point, letter: &str) {
    let rect = Rect::new(at.x, at.y, SWATCH, SWATCH);
    page.outline(rect, RULE_BOX, page::INK_QUIET, TICK_BOX_RADIUS);

    let style = Style::new(Face::MonoBold, SIZE_LEGEND_INITIAL * 0.8);
    let width = text::width(letter, style);
    page.text(
        Point::new(
            rect.x + (SWATCH - width) / 2.0,
            rect.y + SWATCH / 2.0 + text::ascent(style) / 2.0 - 0.35,
        ),
        letter,
        style,
        page::RULE_SUB,
    );
}

/// Sets a run centred on a horizontal line, returning its width.
fn write_middle(
    page: &mut Page,
    x: Mm,
    middle: Mm,
    run: &str,
    style: Style,
    colour: page::Colour,
) -> Mm {
    page.text(
        Point::new(x, middle + text::ascent(style) / 2.0 - 0.3),
        run,
        style,
        colour,
    );
    text::width(run, style)
}

/// The separator that says the stops below it were never given a position.
///
/// The design pass dropped this; `docs/print-spec.md` §6 puts it back, because
/// without it a driver cannot tell "nobody sequenced this" from "this is the
/// last delivery of the day".
pub fn unsequenced_flag_block(column: &Cursor) -> (Page, Mm) {
    let mut own = Page::new();
    let mut cursor = Cursor {
        y: 0.0,
        left: column.left,
        width: column.width,
    };
    let page = &mut own;
    let cursor = &mut cursor;

    cursor.advance(2.4);
    rule(page, cursor, RULE_FOOTER, page::RULE_SUB);
    cursor.advance(1.2);

    let style = micro_label(SIZE_PAGE_NOTE).tracked(TRACK_TAG);
    let height = text_from_top(
        page,
        Point::new(cursor.left, cursor.y),
        "NO POSITION ASSIGNED — DRIVER DECIDES THE ORDER",
        style,
        page::NOTE,
    );
    cursor.advance(height + 0.8);

    (page.clone(), cursor.y)
}

/// Sits at the bottom of the sheet whatever else happened above it.
pub fn footer(page: &mut Page, route: &Route, context: &SheetContext) {
    let y = super::footer_rule_y();
    page.horizontal_rule(
        Point::new(MARGIN_SIDE, y),
        PAGE_WIDTH - 2.0 * MARGIN_SIDE,
        RULE_FOOTER,
        page::RULE_FOOT,
    );

    let style = Style::new(Face::MonoRegular, SIZE_FOOTER);
    let top = y + FOOTER_PADDING_TOP;

    let state = if context.continues() {
        format!(
            "Route {} continues on page {}",
            route.nickname,
            context.page + 1
        )
    } else {
        format!("Route {} — end of route", route.nickname)
    };
    text_from_top(
        page,
        Point::new(MARGIN_SIDE, top),
        &state,
        style,
        page::FAINT,
    );

    let source = format!("{} · Matvare Expressen", context.source);
    text_from_right(
        page,
        PAGE_WIDTH - MARGIN_SIDE,
        top,
        &source,
        style,
        page::FAINT,
    );
}
