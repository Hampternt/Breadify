//! The page's units and its fixed dimensions.
//!
//! Everything the layout decides is in millimetres, measured from the top-left
//! corner of the sheet with `y` growing downwards — the way a page reads. PDF
//! measures in points from the bottom-left corner, and that conversion happens
//! in exactly one place, in the PDF backend.

/// Millimetres. Every position and length on a page.
pub type Mm = f64;

/// Points. Type sizes and rule weights, which the design states in points.
pub type Pt = f64;

/// A4 portrait, the only paper the app prints.
pub const PAGE_WIDTH: Mm = 210.0;
pub const PAGE_HEIGHT: Mm = 297.0;

pub const MARGIN_TOP: Mm = 9.0;
pub const MARGIN_SIDE: Mm = 8.0;
pub const MARGIN_BOTTOM: Mm = 5.0;

/// The width every column budget is carved out of: 194 mm.
pub const CONTENT_WIDTH: Mm = PAGE_WIDTH - 2.0 * MARGIN_SIDE;

/// The product-name column, wide enough for the longest name in the export at
/// the 11 pt floor.
pub const PRODUCT_COLUMN_WIDTH: Mm = 150.0;

/// The gap every page keeps between its last content and the footer. Below
/// this, a printer with slightly different text metrics silently clips a row.
pub const FOOTER_CLEARANCE: Mm = 10.0;

/// A point in millimetres from the top-left of the sheet.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: Mm,
    pub y: Mm,
}

impl Point {
    pub fn new(x: Mm, y: Mm) -> Self {
        Self { x, y }
    }

    /// The same point, moved.
    pub fn offset(self, x: Mm, y: Mm) -> Self {
        Self::new(self.x + x, self.y + y)
    }
}

/// A rectangle in millimetres, anchored at its top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: Mm,
    pub y: Mm,
    pub width: Mm,
    pub height: Mm,
}

impl Rect {
    pub fn new(x: Mm, y: Mm, width: Mm, height: Mm) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> Mm {
        self.x + self.width
    }

    pub fn bottom(&self) -> Mm {
        self.y + self.height
    }
}

/// One point is 1/72 inch, and one inch is 25.4 mm.
///
/// The design handoff quotes the reciprocal rounded to `2.8346 pt` per
/// millimetre; this is that ratio without the rounding, which matters once it
/// is multiplied by a 297 mm page.
pub fn pt_to_mm(points: Pt) -> Mm {
    points * 25.4 / 72.0
}

/// Millimetres back to points, for the PDF's own coordinate space.
pub fn mm_to_pt(millimetres: Mm) -> Pt {
    millimetres * 72.0 / 25.4
}
