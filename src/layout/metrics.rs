//! Every measurement the printed page is built from.
//!
//! Out of the design handoff's geometry and token tables, in one place so a
//! number is never typed twice. Where the decision log has since moved one,
//! the log is the authority and says why: `SIZE_DEPARTMENT` and
//! `SIZE_DPT_TAG` came down from the handoff's 11.9 and 8.6 pt when the
//! department moved under the customer name (**D19**), and
//! `DEPARTMENT_LINE_GAP` and `CRATE_ROW_GAP` are new with **D19** and **D20**
//! — the handoff has no second heading line and no wrapped crate run.
//!
//! Lengths are millimetres, type sizes and rule weights are points.

use crate::geometry::{Mm, Pt};

/// Rule weights, heaviest first.
pub const RULE_TOTAL: Pt = 3.0;
pub const RULE_MASTHEAD: Pt = 2.5;
pub const RULE_DEPARTMENT_BOX: Pt = 1.5;
pub const RULE_BLOCK: Pt = 1.25;
pub const RULE_BOX: Pt = 1.0;
pub const RULE_FOOTER: Pt = 0.5;
pub const RULE_BREAD_LINE: Pt = 0.4;
/// The bar down the side of a block that refuses substitutes.
pub const RULE_NO_SUBSTITUTES: Pt = 5.0;

/// Type sizes.
pub const SIZE_ROUTE_NUMBER: Pt = 25.0;
pub const SIZE_CUSTOMER: Pt = 14.0;
pub const SIZE_QUANTITY: Pt = 13.1;
pub const SIZE_TOTAL_TITLE: Pt = 12.5;
pub const SIZE_DEPARTMENT: Pt = 10.4;
pub const SIZE_PRODUCT: Pt = 11.0;
pub const SIZE_TOTAL_QUANTITY: Pt = 11.0;
pub const SIZE_SUPPLIER_CODE: Pt = 10.7;
pub const SIZE_BADGE: Pt = 10.4;
pub const SIZE_TOTAL_COLUMN_HEAD: Pt = 10.4;
pub const SIZE_LEGEND_INITIAL: Pt = 9.8;
pub const SIZE_MARKER: Pt = 9.2;
pub const SIZE_ROUTE_LABEL: Pt = 9.2;
pub const SIZE_TOTAL_NAME: Pt = 9.2;
pub const SIZE_BOX_LETTER: Pt = 8.9;
pub const SIZE_TOTAL_SUBTOTAL: Pt = 8.9;
pub const SIZE_DPT_TAG: Pt = 7.7;
pub const SIZE_CONTINUED: Pt = 8.6;
pub const SIZE_PAGE_COUNTER: Pt = 8.3;
pub const SIZE_ORDER_ID: Pt = 8.3;
pub const SIZE_TOTAL_META: Pt = 8.3;
pub const SIZE_LEGEND: Pt = 8.0;
pub const SIZE_PAGE_NOTE: Pt = 8.0;
pub const SIZE_DOT_NOTE: Pt = 7.7;
pub const SIZE_FOOTER: Pt = 7.4;

/// Tracking, as a fraction of the type size.
pub const TRACK_ROUTE_NUMBER: f64 = -0.03;
pub const TRACK_CUSTOMER: f64 = -0.02;
pub const TRACK_DEPARTMENT: f64 = -0.015;
pub const TRACK_MICRO_LABEL: f64 = 0.10;
pub const TRACK_TAG: f64 = 0.08;
pub const TRACK_CONTINUED: f64 = 0.06;
pub const TRACK_SUPPLIER_CODE: f64 = 0.02;

/// Masthead.
pub const LOGO_PANEL: (Mm, Mm) = (26.3, 6.3);
pub const LOGO_PADDING: (Mm, Mm) = (1.5, 2.1);
pub const MASTHEAD_RULE_GAP: Mm = 5.9;

/// Legend strip.
pub const LEGEND_PADDING: (Mm, Mm) = (0.9, 1.3);
pub const LEGEND_ITEM_GAP: Mm = 4.0;
pub const SWATCH: Mm = 4.0;

/// Stop block.
pub const BLOCK_PADDING_TOP: Mm = 1.7;
pub const BLOCK_PADDING_BOTTOM: Mm = 1.2;
pub const HEADING_GAP: Mm = 4.6;
pub const CRATE_GAP: Mm = 1.3;
pub const MARKER_GAP: Mm = 2.3;
pub const HEADING_TO_LINES: Mm = 0.7;
pub const NO_SUBSTITUTES_INDENT: Mm = 2.3;

/// Department box. It sits on its own line under the customer name, so the
/// gap above it is what separates the two halves of a heading.
pub const DEPARTMENT_PADDING: (Mm, Mm) = (0.34, 1.3);
pub const DPT_TAG_GAP: Mm = 1.3;
pub const DEPARTMENT_LINE_GAP: Mm = 0.8;

/// Bread line.
pub const LINE_PADDING: (Mm, Mm) = (0.36, 1.3);
pub const LINE_GAP: Mm = 3.6;
pub const TICK_BOX: Mm = 4.6;
pub const TICK_BOX_RADIUS: Mm = 0.35;
pub const TICK_BOX_GAP: Mm = 1.5;
pub const QUANTITY_COLUMN: Mm = 10.1;
pub const SUPPLIER_COLUMN: Mm = 8.0;

/// Crate glyphs.
pub const CRATE_GLYPH: (Mm, Mm) = (7.1, 4.1);
pub const CRATE_GLYPH_GAP: Mm = 1.1;
/// Between rows, when a stop needs more crates than a line will hold.
pub const CRATE_ROW_GAP: Mm = 1.0;
pub const CRATE_GLYPH_RADIUS: Mm = 0.18;

/// Badge.
pub const BADGE_PADDING: (Mm, Mm) = (0.7, 1.6);

/// Route total.
pub const TOTAL_MARGIN_TOP: Mm = 5.0;
pub const TOTAL_PADDING_TOP: Mm = 2.1;
pub const TOTAL_COLUMN_GAP: Mm = 8.4;
pub const TOTAL_QUANTITY_COLUMN: Mm = 8.4;
/// The gap between a total row's quantity and its product name.
pub const TOTAL_NAME_INDENT: Mm = 2.4;
/// How many supplier columns may stand side by side.
///
/// Bread comes from two bakeries and the total was drawn for two. The freezer
/// list has seven suppliers on route 8, and seven columns across the 194 mm
/// page leave 20 mm each — narrower than a product name is long. More than
/// this many wrap onto a further row of columns instead.
pub const TOTAL_COLUMNS_MAX: usize = 2;
/// The gap between one row of supplier columns and the next.
pub const TOTAL_COLUMN_ROW_GAP: Mm = 4.6;
pub const TOTAL_DOT_COLUMN: Mm = 18.0;
pub const TOTAL_DOT: Mm = 2.7;
pub const NOTE_DOT: Mm = 2.5;

/// Footer.
pub const FOOTER_PADDING_TOP: Mm = 1.9;
