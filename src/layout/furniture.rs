//! The page's furniture: masthead, page note, legend strip, the unsequenced
//! flag and the footer.

use super::metrics::*;
use super::{Cursor, micro_label, rule, text_from_right, text_from_top};
use crate::font::Face;
use crate::geometry::{MARGIN_SIDE, MARGIN_TOP, Mm, PAGE_WIDTH, Point, Rect};
use crate::layout::{Settings, SheetContext};
use crate::page::{self, Page};
use crate::route::Route;
use crate::supplier;
use crate::text::{self, Style};

/// The logo panel, the route, the date and the page counter, over the brand
/// rule.
pub fn masthead(page: &mut Page, cursor: &mut Cursor, route: &Route, context: &SheetContext) {
    cursor.y = MARGIN_TOP;
    let top = cursor.y;

    logo_panel(page, Point::new(cursor.left, top));

    let label_left = cursor.left + LOGO_PANEL.0 + 2.0 * LOGO_PADDING.1 + HEADING_GAP;
    let label = Style::new(Face::MonoRegular, SIZE_ROUTE_LABEL).tracked(TRACK_MICRO_LABEL);
    text_from_top(
        page,
        Point::new(label_left, top),
        "ROUTE",
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
pub fn page_note(page: &mut Page, cursor: &mut Cursor, route: &Route, settings: &Settings) {
    let style = Style::new(Face::MonoRegular, SIZE_PAGE_NOTE);
    let unsequenced = route.unsequenced().count();

    // The bread sheet is picked from; the freezer sheet is checked against a
    // box somebody already packed (decision F1), and says so.
    let what = if settings.is_bread() {
        "in full"
    } else {
        "check list"
    };
    let left = match unsequenced {
        0 => format!(
            "Route {} {what} — {} stops.",
            route.nickname,
            route.stops.len()
        ),
        _ => format!(
            "Route {} {what} — {} stops, {unsequenced} with no position assigned.",
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
///
/// The two lists differ here in three ways: the freezer sheet has no crates
/// to explain (decision F4), its `P` means *packed* rather than *picked*, and
/// its suppliers are not the two house bakeries but whichever wholesalers
/// this route actually draws from — so those are read off the route.
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

    let boxes: &[(&str, &str)] = if settings.is_bread() {
        &[("P", "Picked"), ("M", "Missing"), ("F", "Fixed")]
    } else {
        &[("C", "Checked"), ("M", "Missing")]
    };
    x += write_middle(page, x, middle, "BOXES", bold, page::BLACK) + LEGEND_ITEM_GAP;
    for (letter, word) in boxes {
        swatch(page, Point::new(x, middle - SWATCH / 2.0), letter);
        x += SWATCH + 1.0;
        x += write_middle(page, x, middle, word, text_style, page::INK_QUIET) + LEGEND_ITEM_GAP;
    }

    if settings.is_bread() {
        x += write_middle(page, x, middle, "CRATES", bold, page::BLACK) + LEGEND_ITEM_GAP;
        for (full, count) in [(true, "10"), (false, "5")] {
            super::crate_glyph(page, Point::new(x, middle - CRATE_GLYPH.1 / 2.0), full);
            x += CRATE_GLYPH.0 + 1.0;
            x +=
                write_middle(page, x, middle, count, text_style, page::INK_QUIET) + LEGEND_ITEM_GAP;
        }
    }

    let suppliers = supplier_key(
        route,
        settings,
        band.right() - LEGEND_PADDING.1 - x,
        text_style,
    );
    let width = text::width(&suppliers, text_style);
    write_middle(
        page,
        band.right() - LEGEND_PADDING.1 - width,
        middle,
        &suppliers,
        text_style,
        page::INK_QUIET,
    );

    cursor.y = band.bottom() + 2.2;
}

/// The supplier key on the legend's right: code and name for every supplier
/// the sheet's lines can carry.
///
/// The bread list spells out its two bakeries. The freezer list draws from a
/// whole warehouse, so it names only the suppliers on *this route*, in the
/// same order their codes sort elsewhere — and when even those will not fit
/// the room the left of the band has left over, the codes stand alone.
fn supplier_key(route: &Route, settings: &Settings, room: Mm, style: Style) -> String {
    if settings.is_bread() {
        return supplier::KNOWN
            .iter()
            .map(|(_, code, name)| format!("{code} {name}"))
            .collect::<Vec<_>>()
            .join(" · ");
    }

    let mut suppliers: Vec<&str> = route
        .stops
        .iter()
        .flat_map(|stop| &stop.lines)
        .map(|line| line.product.supplier.as_str())
        .collect();
    suppliers.sort_unstable_by_key(|name| supplier::column_position(name));
    suppliers.dedup();

    let spelled = suppliers
        .iter()
        .map(|name| format!("{} {}", supplier::code(name), supplier::display_name(name)))
        .collect::<Vec<_>>()
        .join(" · ");
    if text::width(&spelled, style) <= room {
        return spelled;
    }

    suppliers
        .iter()
        .map(|name| supplier::code(name))
        .collect::<Vec<_>>()
        .join(" · ")
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
