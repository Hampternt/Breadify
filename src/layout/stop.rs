//! One stop: the heading a crate label is copied from, and its bread lines.

use super::metrics::*;
use super::{Cursor, crate_glyph, text_from_top};
use crate::crates::{self, CrateRules};
use crate::font::Face;
use crate::geometry::{Mm, Point, Rect};
use crate::order::{Line, Order};
use crate::page::{self, Page};
use crate::supplier;
use crate::text::{self, Style};

/// Lays out one order — one stop, one crate label, one block.
pub fn block(page: &mut Page, cursor: &mut Cursor, stop: &Order, rules: &CrateRules) {
    let top = cursor.y;
    page.horizontal_rule(
        Point::new(cursor.left, top),
        cursor.width,
        RULE_BLOCK,
        page::BLACK,
    );

    let refuses = !stop.accept_alternatives;
    let left = if refuses {
        cursor.left + NO_SUBSTITUTES_INDENT
    } else {
        cursor.left
    };

    cursor.advance(BLOCK_PADDING_TOP);
    heading(page, cursor, stop, rules, left);
    cursor.advance(HEADING_TO_LINES);

    for (index, line) in stop.lines.iter().enumerate() {
        bread_line(page, cursor, line, left, index % 2 == 1);
    }

    cursor.advance(BLOCK_PADDING_BOTTOM);

    if refuses {
        page.vertical_rule(
            Point::new(cursor.left, top),
            cursor.y - top,
            RULE_NO_SUBSTITUTES,
            page::BLACK,
        );
    }
}

/// Customer, department box, crate glyphs, then the marker and order id on the
/// right of the same line.
///
/// Where the customer name, its department box and its crates will not fit
/// beside that right-hand group — one Stavanger customer has a 32-character
/// name and a 38-character department — the box and the crates drop to a
/// second line rather than run off the sheet. The design handoff has no case
/// long enough to need this; the export does.
fn heading(page: &mut Page, cursor: &mut Cursor, stop: &Order, rules: &CrateRules, left: Mm) {
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

    let order_id = stop.id.to_string();
    let id_style = Style::new(Face::MonoRegular, SIZE_ORDER_ID);
    let id_width = text::width(&order_id, id_style);
    page.text(
        Point::new(
            cursor.right() - id_width,
            middle + text::ascent(id_style) / 2.0 - 0.4,
        ),
        &order_id,
        id_style,
        page::FAINTEST,
    );

    let marker_right = cursor.right() - id_width - MARKER_GAP;
    marker(page, marker_right, middle, stop);

    let count = crates::count(stop, rules);
    let crates_width = crate_run_width(count.large + count.small);
    let department_width = stop
        .department
        .as_ref()
        .map_or(0.0, |department| department_box_width(department, height));

    let after_name = left + text::width(&stop.customer, name);
    let wanted = after_name
        + if department_width > 0.0 {
            HEADING_GAP + department_width
        } else {
            0.0
        }
        + if crates_width > 0.0 {
            CRATE_GAP + crates_width
        } else {
            0.0
        };
    let available = marker_right - marker_width(stop) - MARKER_GAP;

    if wanted <= available {
        let mut x = after_name;
        if let Some(department) = &stop.department {
            x = department_box(page, Point::new(x + HEADING_GAP, top), department, height);
        }
        draw_crates(page, x + CRATE_GAP, middle, count);
        cursor.advance(height);
        return;
    }

    cursor.advance(height + 0.4);
    let second = cursor.y;
    let mut x = left;
    if let Some(department) = &stop.department {
        x = department_box(page, Point::new(x, second), department, height);
    }
    draw_crates(page, x + CRATE_GAP, second + height / 2.0, count);
    cursor.advance(height);
}

/// How wide a row of crate glyphs is.
fn crate_run_width(crates: u32) -> Mm {
    if crates == 0 {
        return 0.0;
    }
    f64::from(crates) * (CRATE_GLYPH.0 + CRATE_GLYPH_GAP) - CRATE_GLYPH_GAP
}

/// Draws a stop's crates, full ones first.
fn draw_crates(page: &mut Page, mut x: Mm, middle: Mm, count: crates::CrateCount) {
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
fn marker_width(stop: &Order) -> Mm {
    if stop.accept_alternatives {
        return text::width(
            "want substitute: true",
            Style::new(Face::MonoMedium, SIZE_MARKER),
        );
    }
    text::width(
        "WANT SUBSTITUTE: FALSE",
        Style::new(Face::ArchivoExtraBold, SIZE_BADGE).tracked(TRACK_CUSTOMER),
    ) + 2.0 * BADGE_PADDING.1
}

/// How wide the department box will be, before drawing it.
fn department_box_width(department: &str, line_height: Mm) -> Mm {
    let _ = line_height;
    let tag = Style::new(Face::MonoBold, SIZE_DPT_TAG).tracked(TRACK_TAG);
    let name = Style::new(Face::ArchivoExtraBold, SIZE_DEPARTMENT).tracked(TRACK_DEPARTMENT);
    text::width("DPT", tag)
        + DPT_TAG_GAP * 2.0
        + text::width(department, name)
        + 2.0 * DEPARTMENT_PADDING.1
}

/// The crate label: a `DPT` tag and the department name in a hard-ruled box.
fn department_box(page: &mut Page, at: Point, department: &str, line_height: Mm) -> Mm {
    let tag = Style::new(Face::MonoBold, SIZE_DPT_TAG).tracked(TRACK_TAG);
    let name = Style::new(Face::ArchivoExtraBold, SIZE_DEPARTMENT).tracked(TRACK_DEPARTMENT);

    let tag_width = text::width("DPT", tag);
    let name_width = text::width(department, name);
    let inner = tag_width + DPT_TAG_GAP * 2.0 + name_width;
    let height = line_height + 2.0 * DEPARTMENT_PADDING.0;
    let box_rect = Rect::new(
        at.x,
        at.y - DEPARTMENT_PADDING.0,
        inner + 2.0 * DEPARTMENT_PADDING.1,
        height,
    );

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

    box_rect.right()
}

/// Quiet when substitutes are fine, inverted and loud when they are not.
fn marker(page: &mut Page, right: Mm, middle: Mm, stop: &Order) {
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
