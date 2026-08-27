//! One stop: the heading a crate label is copied from, and its bread lines.

use super::metrics::*;
use super::{Cursor, crate_glyph, text_from_top};
use crate::crates::{self};
use crate::font::Face;
use crate::geometry::{Mm, Point, Rect};
use crate::layout::Settings;
use crate::order::{Line, Order};
use crate::page::{self, Page};
use crate::supplier;
use crate::text::{self, Style};

/// Lays out one order — one stop, one crate label, one block — on a page of
/// its own, starting at its top edge.
///
/// The sheet then places it with [`Page::absorb`](crate::page::Page::absorb).
/// Laying a block out before knowing where it goes is what lets the paginator
/// ask how tall it is without a second formula that could disagree with the
/// drawing.
pub fn block(stop: &Order, settings: &Settings, column: &Cursor) -> (Page, Mm) {
    let mut page = Page::new();
    let mut own = Cursor {
        y: 0.0,
        left: column.left,
        width: column.width,
    };
    let cursor = &mut own;
    let page = &mut page;

    let top = cursor.y;
    page.horizontal_rule(
        Point::new(cursor.left, top),
        cursor.width,
        RULE_BLOCK,
        page::BLACK,
    );

    let refuses = !stop.accept_alternatives;
    let barred = refuses && settings.marker.has_rule();
    let left = if barred {
        cursor.left + NO_SUBSTITUTES_INDENT
    } else {
        cursor.left
    };

    cursor.advance(BLOCK_PADDING_TOP);
    heading(page, cursor, stop, settings, left);
    cursor.advance(HEADING_TO_LINES);

    for (index, line) in stop.lines.iter().enumerate() {
        bread_line(page, cursor, line, left, index % 2 == 1);
    }

    cursor.advance(BLOCK_PADDING_BOTTOM);

    if barred {
        page.vertical_rule(
            Point::new(cursor.left, top),
            cursor.y - top,
            RULE_NO_SUBSTITUTES,
            page::BLACK,
        );
    }

    (page.clone(), cursor.y)
}

/// The customer name and, on the right of the same line, the crate glyphs,
/// the substitute marker and the order id.
///
/// A department gets a line of its own beneath the name, set smaller, so the
/// heading reads as one crate label in two parts rather than a row of equals.
/// The crates keep to the right-hand group because that is where the picker's
/// eye already is for the marker; where a long customer name would run into
/// them — one Stavanger name is 127 mm at 14 pt — they drop to the second line
/// rather than collide.
fn heading(page: &mut Page, cursor: &mut Cursor, stop: &Order, settings: &Settings, left: Mm) {
    let name = Style::new(Face::ArchivoExtraBold, SIZE_CUSTOMER).tracked(TRACK_CUSTOMER);
    let top = cursor.y;
    let height = text_from_top(
        page,
        Point::new(left, top),
        &stop.customer,
        name,
        page::BLACK,
    );
    let middle = top + height / 2.0;

    let id_style = Style::new(Face::MonoRegular, SIZE_ORDER_ID);
    let id_width = if settings.show_order_id {
        let order_id = stop.id.to_string();
        let width = text::width(&order_id, id_style);
        page.text(
            Point::new(
                cursor.right() - width,
                middle + text::ascent(id_style) / 2.0 - 0.4,
            ),
            &order_id,
            id_style,
            page::FAINTEST,
        );
        width + MARKER_GAP
    } else {
        0.0
    };

    let marker_right = cursor.right() - id_width;
    marker(page, marker_right, middle, stop, settings);

    let count = crates::count(stop, &settings.crates);
    let crates_width = crate_run_width(count.total());
    let crates_left = marker_right - marker_width(stop, settings) - MARKER_GAP - crates_width;
    let name_right = left + text::width(&stop.customer, name);

    let beside_the_marker = crates_width == 0.0 || crates_left >= name_right + CRATE_GAP;
    if beside_the_marker && crates_width > 0.0 {
        draw_crates(page, crates_left, middle, count);
    }
    cursor.advance(height);

    let second_height = match (&stop.department, beside_the_marker) {
        (None, true) => return,
        (None, false) => CRATE_GLYPH.1,
        (Some(_), true) => department_box_height(),
        (Some(_), false) => department_box_height().max(CRATE_GLYPH.1),
    };

    cursor.advance(DEPARTMENT_LINE_GAP);
    let second = cursor.y;
    if let Some(department) = &stop.department {
        department_box(page, Point::new(left, second), department);
    }
    if !beside_the_marker {
        draw_crates(
            page,
            marker_right - crates_width,
            second + second_height / 2.0,
            count,
        );
    }
    cursor.advance(second_height);
}

/// How wide a row of crate glyphs is.
fn crate_run_width(crates: u32) -> Mm {
    if crates == 0 {
        return 0.0;
    }
    f64::from(crates) * (CRATE_GLYPH.0 + CRATE_GLYPH_GAP) - CRATE_GLYPH_GAP
}

/// Draws a stop's crates, full ones first.
fn draw_crates(page: &mut Page, mut x: Mm, middle: Mm, count: crate::crates::CrateCount) {
    for index in 0..count.large + count.small {
        crate_glyph(
            page,
            Point::new(x, middle - CRATE_GLYPH.1 / 2.0),
            index < count.large,
        );
        x += CRATE_GLYPH.0 + CRATE_GLYPH_GAP;
    }
}

/// How much room the marker takes, so the heading knows what is left.
fn marker_width(stop: &Order, settings: &Settings) -> Mm {
    if stop.accept_alternatives {
        return text::width(
            "want substitute: true",
            Style::new(Face::MonoMedium, SIZE_MARKER),
        );
    }

    let words = text::width(
        "WANT SUBSTITUTE: FALSE",
        Style::new(Face::ArchivoExtraBold, SIZE_BADGE).tracked(TRACK_CUSTOMER),
    );
    if settings.marker.has_badge() {
        return words + 2.0 * BADGE_PADDING.1;
    }
    words
}

/// How wide the department box will be. [`department_box`] draws exactly this
/// width, so the heading's fit test and the drawing cannot drift apart.
fn department_box_width(department: &str) -> Mm {
    let tag = Style::new(Face::MonoBold, SIZE_DPT_TAG).tracked(TRACK_TAG);
    let name = Style::new(Face::ArchivoExtraBold, SIZE_DEPARTMENT).tracked(TRACK_DEPARTMENT);
    text::width("DPT", tag)
        + DPT_TAG_GAP * 2.0
        + text::width(department, name)
        + 2.0 * DEPARTMENT_PADDING.1
}

/// How tall it will be. On a line of its own the box is sized by its own type,
/// not by the customer name above it.
fn department_box_height() -> Mm {
    let name = Style::new(Face::ArchivoExtraBold, SIZE_DEPARTMENT).tracked(TRACK_DEPARTMENT);
    text::line_height(name) + 2.0 * DEPARTMENT_PADDING.0
}

/// The crate label: a `DPT` tag and the department name in a hard-ruled box,
/// with `at` its top-left corner.
fn department_box(page: &mut Page, at: Point, department: &str) {
    let tag = Style::new(Face::MonoBold, SIZE_DPT_TAG).tracked(TRACK_TAG);
    let name = Style::new(Face::ArchivoExtraBold, SIZE_DEPARTMENT).tracked(TRACK_DEPARTMENT);

    let tag_width = text::width("DPT", tag);
    let height = department_box_height();
    let box_rect = Rect::new(at.x, at.y, department_box_width(department), height);

    page.outline(box_rect, RULE_DEPARTMENT_BOX, page::BLACK, 0.0);

    let middle = box_rect.y + height / 2.0;
    page.text(
        Point::new(
            box_rect.x + DEPARTMENT_PADDING.1,
            middle + text::ascent(tag) / 2.0 - 0.35,
        ),
        "DPT",
        tag,
        page::BLACK,
    );

    let divider = box_rect.x + DEPARTMENT_PADDING.1 + tag_width + DPT_TAG_GAP;
    page.vertical_rule(
        Point::new(divider, box_rect.y),
        height,
        RULE_BOX,
        page::BLACK,
    );

    page.text(
        Point::new(
            divider + DPT_TAG_GAP,
            middle + text::ascent(name) / 2.0 - 0.4,
        ),
        department,
        name,
        page::BLACK,
    );
}

/// Quiet when substitutes are fine, inverted and loud when they are not.
fn marker(page: &mut Page, right: Mm, middle: Mm, stop: &Order, settings: &Settings) {
    if stop.accept_alternatives {
        let style = Style::new(Face::MonoMedium, SIZE_MARKER);
        let run = "want substitute: true";
        let width = text::width(run, style);
        page.text(
            Point::new(right - width, middle + text::ascent(style) / 2.0 - 0.4),
            run,
            style,
            page::INK_QUIET,
        );
        return;
    }

    let style = Style::new(Face::ArchivoExtraBold, SIZE_BADGE).tracked(TRACK_CUSTOMER);
    let run = "WANT SUBSTITUTE: FALSE";
    let width = text::width(run, style);

    if !settings.marker.has_badge() {
        page.text(
            Point::new(right - width, middle + text::ascent(style) / 2.0 - 0.4),
            run,
            style,
            page::BLACK,
        );
        return;
    }

    let height = text::line_height(style) + 2.0 * BADGE_PADDING.0;
    let badge = Rect::new(
        right - width - 2.0 * BADGE_PADDING.1,
        middle - height / 2.0,
        width + 2.0 * BADGE_PADDING.1,
        height,
    );

    page.fill(badge, page::BLACK);
    page.text(
        Point::new(
            badge.x + BADGE_PADDING.1,
            middle + text::ascent(style) / 2.0 - 0.4,
        ),
        run,
        style,
        page::WHITE,
    );
}

/// Tick box, quantity, supplier code, product name, then the missing and fixed
/// boxes — with every second line tinted.
fn bread_line(page: &mut Page, cursor: &mut Cursor, line: &Line, left: Mm, tinted: bool) {
    let name = Style::new(Face::SpaceGrotesk, SIZE_PRODUCT);
    let height = TICK_BOX.max(text::line_height(name)) + 2.0 * LINE_PADDING.0;
    let row = Rect::new(left, cursor.y, cursor.right() - left, height);

    if tinted {
        page.fill(row, page::ZEBRA);
    }

    let middle = row.y + height / 2.0;
    let mut x = row.x + LINE_PADDING.1;

    tick_box(page, Point::new(x, middle - TICK_BOX / 2.0), "P");
    x += TICK_BOX + LINE_GAP;

    let quantity = Style::new(Face::MonoSemiBold, SIZE_QUANTITY);
    let quantity_text = line.quantity.to_string();
    let quantity_width = text::width(&quantity_text, quantity);
    page.text(
        Point::new(
            x + QUANTITY_COLUMN - quantity_width,
            middle + text::ascent(quantity) / 2.0 - 0.5,
        ),
        &quantity_text,
        quantity,
        page::BLACK,
    );
    x += QUANTITY_COLUMN + LINE_GAP;

    let code = Style::new(Face::MonoBold, SIZE_SUPPLIER_CODE).tracked(TRACK_SUPPLIER_CODE);
    page.text(
        Point::new(x, middle + text::ascent(code) / 2.0 - 0.45),
        supplier::code(&line.product.supplier),
        code,
        page::BLACK,
    );
    x += SUPPLIER_COLUMN + LINE_GAP;

    page.text(
        Point::new(x, middle + text::ascent(name) / 2.0 - 0.45),
        &line.product.name,
        name,
        page::BLACK,
    );

    let mut boxes = row.right() - LINE_PADDING.1 - TICK_BOX;
    for letter in ["F", "M"] {
        tick_box(page, Point::new(boxes, middle - TICK_BOX / 2.0), letter);
        boxes -= TICK_BOX + TICK_BOX_GAP;
    }

    page.horizontal_rule(
        Point::new(row.x, row.bottom()),
        row.width,
        RULE_BREAD_LINE,
        page::RULE_LINE,
    );
    cursor.advance(height);
}

/// An empty box for the picker's pen, with its letter set faintly inside so it
/// says what it is without reading as ticked.
fn tick_box(page: &mut Page, at: Point, letter: &str) {
    let rect = Rect::new(at.x, at.y, TICK_BOX, TICK_BOX);
    page.outline(rect, RULE_BOX, page::INK_QUIET, TICK_BOX_RADIUS);

    let style = Style::new(Face::MonoBold, SIZE_BOX_LETTER);
    let width = text::width(letter, style);
    page.text(
        Point::new(
            rect.x + (TICK_BOX - width) / 2.0,
            rect.y + TICK_BOX / 2.0 + text::ascent(style) / 2.0 - 0.4,
        ),
        letter,
        style,
        page::RULE_SUB,
    );
}
