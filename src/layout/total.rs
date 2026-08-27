//! The route total that closes a route's last page.

use super::metrics::*;
use super::{Cursor, rule, text_from_top};
use crate::font::Face;
use crate::geometry::{Mm, Point, Rect};
use crate::page::{self, Page};
use crate::route::Route;
use crate::supplier;
use crate::text::{self, Style};
use crate::total::{self, SupplierColumn};

/// Lays out the whole total — title, the two meta lines, then one column per
/// bakery — on a page of its own, the way a stop block is laid out.
pub fn block(route: &Route, column: &Cursor) -> (Page, Mm) {
    let total = total::of(route);
    let mut own = Page::new();
    let mut cursor = Cursor {
        y: 0.0,
        left: column.left,
        width: column.width,
    };
    let page = &mut own;
    let cursor = &mut cursor;

    cursor.advance(TOTAL_MARGIN_TOP);
    rule(page, cursor, RULE_TOTAL, page::BLACK);
    cursor.advance(TOTAL_PADDING_TOP);

    let title = Style::new(Face::ArchivoExtraBold, SIZE_TOTAL_TITLE).tracked(TRACK_CUSTOMER);
    let height = text_from_top(
        page,
        Point::new(cursor.left, cursor.y),
        &format!("Route {} total", route.nickname),
        title,
        page::BLACK,
    );
    cursor.advance(height + 0.6);

    let meta = Style::new(Face::MonoRegular, SIZE_TOTAL_META);
    let summary = format!(
        "{} bread {} · {} {} · most to least",
        total.types(),
        plural(total.types() as u32, "type"),
        total.units(),
        plural(total.units(), "unit")
    );
    let height = text_from_top(
        page,
        Point::new(cursor.left, cursor.y),
        &summary,
        meta,
        page::NOTE,
    );
    cursor.advance(height + 0.4);

    dot_note(page, cursor, total.full_tens());
    columns(page, cursor, &total.columns);

    (page.clone(), cursor.y)
}

/// The flat total that closes a freezer route: one list, most needed first,
/// in two balanced columns — no bakery columns, no ten-dots, and no supplier
/// code either: that cue lives on the stop lines, and the longest freezer
/// names need the room here (decision F9).
pub fn check_block(route: &Route, column: &Cursor) -> (Page, Mm) {
    let lines = total::flat(route);
    let units: u32 = lines.iter().map(|line| line.units).sum();
    let mut own = Page::new();
    let mut cursor = Cursor {
        y: 0.0,
        left: column.left,
        width: column.width,
    };
    let page = &mut own;
    let cursor = &mut cursor;

    cursor.advance(TOTAL_MARGIN_TOP);
    rule(page, cursor, RULE_TOTAL, page::BLACK);
    cursor.advance(TOTAL_PADDING_TOP);

    let title = Style::new(Face::ArchivoExtraBold, SIZE_TOTAL_TITLE).tracked(TRACK_CUSTOMER);
    let height = text_from_top(
        page,
        Point::new(cursor.left, cursor.y),
        &format!("Route {} total", route.nickname),
        title,
        page::BLACK,
    );
    cursor.advance(height + 0.6);

    let meta = Style::new(Face::MonoRegular, SIZE_TOTAL_META);
    let summary = format!("{} · most to least", total::summary(lines.len(), units));
    let height = text_from_top(
        page,
        Point::new(cursor.left, cursor.y),
        &summary,
        meta,
        page::NOTE,
    );
    cursor.advance(height + 1.8);

    // Read down the first column, then down the second. A one-line route
    // keeps the full width rather than an empty twin.
    let split = lines.len().div_ceil(2);
    let count: f64 = if lines.len() > 1 { 2.0 } else { 1.0 };
    let width = (cursor.width - TOTAL_COLUMN_GAP * (count - 1.0)) / count;
    let top = cursor.y;
    let mut lowest = top;

    for (index, half) in [&lines[..split], &lines[split..]].into_iter().enumerate() {
        let left = cursor.left + (width + TOTAL_COLUMN_GAP) * index as f64;
        let mut y = top;
        for line in half {
            y = check_row(page, Point::new(left, y), width, line);
        }
        lowest = lowest.max(y);
    }
    cursor.y = lowest;

    (page.clone(), cursor.y)
}

/// A row of the flat total: how many, and of what.
fn check_row(page: &mut Page, at: Point, width: Mm, line: &total::TotalLine) -> Mm {
    let quantity = Style::new(Face::MonoSemiBold, SIZE_TOTAL_QUANTITY);
    let name = Style::new(Face::SpaceGrotesk, SIZE_TOTAL_NAME);
    let height = text::line_height(quantity) + 0.7;
    let middle = at.y + height / 2.0;

    let units = line.units.to_string();
    let units_width = text::width(&units, quantity);
    page.text(
        Point::new(
            at.x + TOTAL_QUANTITY_COLUMN - units_width,
            middle + text::ascent(quantity) / 2.0 - 0.45,
        ),
        &units,
        quantity,
        page::BLACK,
    );

    page.text(
        Point::new(
            at.x + TOTAL_QUANTITY_COLUMN + 2.4,
            middle + text::ascent(name) / 2.0 - 0.4,
        ),
        &line.product.name,
        name,
        page::INK_SOFT,
    );

    page.horizontal_rule(
        Point::new(at.x, at.y + height),
        width,
        RULE_BREAD_LINE,
        page::RULE_LINE,
    );
    at.y + height
}

/// What a ten-dot means, and how many the route has.
fn dot_note(page: &mut Page, cursor: &mut Cursor, dots: u32) {
    let style = Style::new(Face::MonoRegular, SIZE_DOT_NOTE);
    let height = text::line_height(style);
    let middle = cursor.y + height / 2.0;

    page.fill(
        Rect::new(cursor.left, middle - NOTE_DOT / 2.0, NOTE_DOT, NOTE_DOT),
        page::BLACK,
    );

    let note = format!("one full ten inside a single order — {dots} on this route");
    page.text(
        Point::new(
            cursor.left + NOTE_DOT + 1.2,
            middle + text::ascent(style) / 2.0 - 0.35,
        ),
        &note,
        style,
        page::INK_QUIET,
    );
    cursor.advance(height + 1.8);
}

/// One column per bakery, side by side, each as tall as it needs to be.
fn columns(page: &mut Page, cursor: &mut Cursor, columns: &[SupplierColumn]) {
    if columns.is_empty() {
        return;
    }

    let count = columns.len() as f64;
    let width = (cursor.width - TOTAL_COLUMN_GAP * (count - 1.0)) / count;
    let top = cursor.y;
    let mut lowest = top;

    for (index, column) in columns.iter().enumerate() {
        let left = cursor.left + (width + TOTAL_COLUMN_GAP) * index as f64;
        let bottom = supplier_column(page, Point::new(left, top), width, column);
        lowest = lowest.max(bottom);
    }

    cursor.y = lowest;
}

/// A column: its head, then its breads most needed first.
fn supplier_column(page: &mut Page, at: Point, width: Mm, column: &SupplierColumn) -> Mm {
    let code = Style::new(Face::MonoBold, SIZE_SUPPLIER_CODE).tracked(TRACK_SUPPLIER_CODE);
    let name = Style::new(Face::ArchivoExtraBold, SIZE_TOTAL_COLUMN_HEAD).tracked(TRACK_CUSTOMER);
    let subtotal = Style::new(Face::MonoMedium, SIZE_TOTAL_SUBTOTAL);

    let head_height = text::line_height(name);
    let middle = at.y + head_height / 2.0;

    page.text(
        Point::new(at.x, middle + text::ascent(code) / 2.0 - 0.45),
        supplier::code(&column.supplier),
        code,
        page::BLACK,
    );
    page.text(
        Point::new(
            at.x + text::width("SB", code) + 1.6,
            middle + text::ascent(name) / 2.0 - 0.45,
        ),
        supplier::display_name(&column.supplier),
        name,
        page::BLACK,
    );

    let counts = total::summary(column.types(), column.units());
    let counts_width = text::width(&counts, subtotal);
    page.text(
        Point::new(
            at.x + width - counts_width,
            middle + text::ascent(subtotal) / 2.0 - 0.4,
        ),
        &counts,
        subtotal,
        page::LABEL,
    );

    let mut y = at.y + head_height + 0.5;
    page.horizontal_rule(Point::new(at.x, y), width, RULE_BLOCK, page::BLACK);
    y += 0.8;

    for line in &column.lines {
        y = total_row(page, Point::new(at.x, y), width, line);
    }
    y
}

/// A row of the total: how many, of what, and its full trays.
fn total_row(page: &mut Page, at: Point, width: Mm, line: &crate::total::TotalLine) -> Mm {
    let quantity = Style::new(Face::MonoSemiBold, SIZE_TOTAL_QUANTITY);
    let name = Style::new(Face::SpaceGrotesk, SIZE_TOTAL_NAME);
    let height = text::line_height(quantity) + 0.7;
    let middle = at.y + height / 2.0;

    let units = line.units.to_string();
    let units_width = text::width(&units, quantity);
    page.text(
        Point::new(
            at.x + TOTAL_QUANTITY_COLUMN - units_width,
            middle + text::ascent(quantity) / 2.0 - 0.45,
        ),
        &units,
        quantity,
        page::BLACK,
    );

    page.text(
        Point::new(
            at.x + TOTAL_QUANTITY_COLUMN + 2.4,
            middle + text::ascent(name) / 2.0 - 0.4,
        ),
        &line.product.name,
        name,
        page::INK_SOFT,
    );

    dots(page, at.x + width, middle, line.full_tens);

    page.horizontal_rule(
        Point::new(at.x, at.y + height),
        width,
        RULE_BREAD_LINE,
        page::RULE_LINE,
    );
    at.y + height
}

/// The full trays, drawn right to left in their own column.
///
/// More trays than the column holds — no route in the sample has that many,
/// but a bigger day would — are written as a count beside a single dot rather
/// than run into the product name.
fn dots(page: &mut Page, right: Mm, middle: Mm, full_tens: u32) {
    if full_tens == 0 {
        return;
    }

    let pitch = TOTAL_DOT + 1.0;
    let room = ((TOTAL_DOT_COLUMN + 1.0) / pitch).floor() as u32;

    if full_tens <= room {
        for index in 0..full_tens {
            page.fill(
                Rect::new(
                    right - TOTAL_DOT - f64::from(index) * pitch,
                    middle - TOTAL_DOT / 2.0,
                    TOTAL_DOT,
                    TOTAL_DOT,
                ),
                page::BLACK,
            );
        }
        return;
    }

    page.fill(
        Rect::new(
            right - TOTAL_DOT,
            middle - TOTAL_DOT / 2.0,
            TOTAL_DOT,
            TOTAL_DOT,
        ),
        page::BLACK,
    );
    let style = Style::new(Face::MonoSemiBold, SIZE_DOT_NOTE);
    let count = format!("×{full_tens}");
    page.text(
        Point::new(
            right - TOTAL_DOT - 1.0 - text::width(&count, style),
            middle + text::ascent(style) / 2.0 - 0.35,
        ),
        &count,
        style,
        page::BLACK,
    );
}

fn plural(count: u32, word: &str) -> String {
    if count == 1 {
        return word.to_owned();
    }
    format!("{word}s")
}
