//! Reading the export's one worksheet into raw rows.
//!
//! The file is an ordinary `.xlsx` with a single sheet named `Data`: 14
//! headers in `A1:N1` and a fifteenth column that carries data in every row
//! but has no header. Empty cells are *absent* rather than blank, so
//! `Department` and `Comment` are read as `Option<String>` and everything
//! else is required. See `docs/excel-format.md`.

use std::path::Path;

use calamine::{Data, Range, Reader, Xlsx, open_workbook};

/// The sheet every export carries its data on.
pub const SHEET_NAME: &str = "Data";

/// The 14 headers in `A1:N1`, in order. A fifteenth column follows them with
/// data but no header, which is why this is shorter than [`COLUMN_COUNT`].
pub const HEADERS: [&str; 14] = [
    "Order ID",
    "Quantity",
    "Product ID",
    "Product Name",
    "Supplier SKU",
    "Position",
    "Supplier",
    "Customer",
    "Department",
    "Delivery street",
    "Comment",
    "Route nickname",
    "Route ordering",
    "Accept alternatives",
];

/// Columns `A` through `O` — one more than there are headers.
pub const COLUMN_COUNT: usize = 15;

/// One row of the export: a single order line, one product for one order.
///
/// Everything except [`quantity`](Self::quantity),
/// [`product_id`](Self::product_id) and the product's names is an attribute of
/// the *order*, repeated onto each of its lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetRow {
    /// 1-based row number in the worksheet, for pointing at a row in a
    /// validation finding.
    pub excel_row: usize,
    pub order_id: i64,
    pub quantity: u32,
    pub product_id: u32,
    pub product_name: String,
    /// Text, never a number: `107_san`, `10022_bhb`, bare `115`, `21061bhb`.
    pub supplier_sku: String,
    /// Where the goods sit. On a bread export it is the supplier again,
    /// spelled `X-Sandnes Bakeri` / `X-Bakehuset`; on a freezer export it is a
    /// warehouse shelf — `W-05-02`, `U-Frysevare` — and 26 of 231 rows have no
    /// cell at all. Empty means the export gave none. Nothing is grouped by it
    /// and nothing prints it (decision D4).
    pub position: String,
    pub supplier: String,
    pub customer: String,
    /// The sub-location inside a customer, and the crate label. Absent on
    /// about three quarters of rows.
    pub department: Option<String>,
    pub delivery_street: String,
    /// Order-level free text, repeated onto every line of that order.
    pub comment: Option<String>,
    /// Text even when it looks numeric: `1`..`14`, `hau 1`, `hau 2`.
    pub route_nickname: String,
    /// Stop sequence within the route, higher being later. `0` means no
    /// position was assigned, not position zero (decision D3).
    pub route_ordering: u32,
    /// Whether the customer accepts a replacement product when a bread is
    /// sold out. A real Excel boolean in the file.
    pub accept_alternatives: bool,
    /// The unlabelled fifteenth column. `Stavanger` in every row of every
    /// export seen so far; carried through, never keyed off.
    pub region: String,
}

/// Everything that can stop an export being read at all.
///
/// Problems *within* a readable file — a route that appears at two addresses,
/// an order whose lines disagree — are validation findings rather than errors;
/// they do not belong here.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("could not open {path}: {source}")]
    Open {
        path: String,
        #[source]
        source: calamine::XlsxError,
    },

    #[error("the workbook has no sheet named {SHEET_NAME:?}")]
    NoDataSheet,

    #[error("the sheet is empty")]
    EmptySheet,

    #[error(
        "the sheet does not start at cell A1 (it starts at row {row}, column {column}), \
         so the columns cannot be trusted"
    )]
    NotAnchoredAtA1 { row: u32, column: u32 },

    #[error("expected {COLUMN_COUNT} columns (A to O) but the sheet has {found}")]
    WrongColumnCount { found: usize },

    #[error("cell {column}1 should be the header {expected:?} but reads {found:?}")]
    WrongHeader {
        column: char,
        expected: &'static str,
        found: String,
    },

    #[error(
        "cell {column}1 should be empty — the fifteenth column has no header — but reads {found:?}"
    )]
    UnexpectedHeader { column: char, found: String },

    #[error("cell {column}{row} should hold {expected} but reads {found:?}")]
    WrongCellType {
        column: char,
        row: usize,
        expected: &'static str,
        found: String,
    },
}

/// Reads every data row of `path`'s `Data` sheet, in the order the file has
/// them.
///
/// # Errors
///
/// Fails if the file cannot be opened, if the sheet is missing or misshapen,
/// or if any cell holds something other than what its column calls for.
pub fn read(path: &Path) -> Result<Vec<SheetRow>, ReadError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|source| ReadError::Open {
        path: path.display().to_string(),
        source,
    })?;

    let range = workbook
        .worksheet_range(SHEET_NAME)
        .map_err(|_| ReadError::NoDataSheet)?;

    check_shape(&range)?;
    check_headers(&range)?;

    range
        .rows()
        .enumerate()
        .skip(1)
        .map(|(index, cells)| read_row(index + 1, cells))
        .collect()
}

/// Fails unless the used range starts at `A1` and is exactly `COLUMN_COUNT`
/// wide, which is what makes a positional read of each row safe.
fn check_shape(range: &Range<Data>) -> Result<(), ReadError> {
    let Some((row, column)) = range.start() else {
        return Err(ReadError::EmptySheet);
    };
    if (row, column) != (0, 0) {
        return Err(ReadError::NotAnchoredAtA1 { row, column });
    }
    if range.width() != COLUMN_COUNT {
        return Err(ReadError::WrongColumnCount {
            found: range.width(),
        });
    }
    Ok(())
}

/// Fails unless row 1 carries [`HEADERS`] in `A1:N1` and nothing in `O1`.
fn check_headers(range: &Range<Data>) -> Result<(), ReadError> {
    let Some(header_row) = range.rows().next() else {
        return Err(ReadError::EmptySheet);
    };

    for (index, expected) in HEADERS.iter().enumerate() {
        let found = header_row[index].to_string();
        if found != *expected {
            return Err(ReadError::WrongHeader {
                column: column_letter(index),
                expected,
                found,
            });
        }
    }

    let trailing = &header_row[HEADERS.len()];
    if !matches!(trailing, Data::Empty) {
        return Err(ReadError::UnexpectedHeader {
            column: column_letter(HEADERS.len()),
            found: trailing.to_string(),
        });
    }

    Ok(())
}

fn read_row(excel_row: usize, cells: &[Data]) -> Result<SheetRow, ReadError> {
    let at = |index: usize| Cell {
        data: &cells[index],
        column: column_letter(index),
        row: excel_row,
    };

    Ok(SheetRow {
        excel_row,
        order_id: at(0).integer()?,
        quantity: at(1).count()?,
        product_id: at(2).count()?,
        product_name: at(3).text()?,
        supplier_sku: at(4).text()?,
        position: at(5).optional_text()?.unwrap_or_default(),
        supplier: at(6).text()?,
        customer: at(7).text()?,
        department: at(8).optional_text()?,
        delivery_street: at(9).text()?,
        comment: at(10).optional_text()?,
        route_nickname: at(11).text()?,
        route_ordering: at(12).count()?,
        accept_alternatives: at(13).boolean()?,
        region: at(14).text()?,
    })
}

/// One cell, together with where it sits, so a type mismatch can name itself.
struct Cell<'a> {
    data: &'a Data,
    column: char,
    row: usize,
}

impl Cell<'_> {
    fn wrong(&self, expected: &'static str) -> ReadError {
        ReadError::WrongCellType {
            column: self.column,
            row: self.row,
            expected,
            found: self.data.to_string(),
        }
    }

    /// A required string cell. Excel stores these as shared strings, so a
    /// number here means the export changed shape.
    fn text(&self) -> Result<String, ReadError> {
        match self.data {
            Data::String(value) => Ok(value.trim().to_owned()),
            _ => Err(self.wrong("text")),
        }
    }

    /// A string cell that may be absent — `Department` and `Comment` have no
    /// cell at all when they are unset.
    fn optional_text(&self) -> Result<Option<String>, ReadError> {
        match self.data {
            Data::Empty => Ok(None),
            Data::String(value) => Ok(Some(value.trim().to_owned())),
            _ => Err(self.wrong("text or nothing")),
        }
    }

    /// A whole number. Excel stores these as floats, so `1000620628` arrives
    /// as `1000620628.0` and has to be recognised as the integer it is.
    fn integer(&self) -> Result<i64, ReadError> {
        match self.data {
            Data::Int(value) => Ok(*value),
            Data::Float(value) if value.fract() == 0.0 => Ok(*value as i64),
            _ => Err(self.wrong("a whole number")),
        }
    }

    /// A whole number that cannot be negative: a quantity, an identifier, a
    /// stop sequence.
    fn count(&self) -> Result<u32, ReadError> {
        let value = self.integer()?;
        u32::try_from(value).map_err(|_| self.wrong("a whole number of zero or more"))
    }

    /// A real Excel boolean. `Accept alternatives` is stored as one, so
    /// reading it as `0`/`1` would miss it.
    fn boolean(&self) -> Result<bool, ReadError> {
        match self.data {
            Data::Bool(value) => Ok(*value),
            _ => Err(self.wrong("true or false")),
        }
    }
}

/// `0` -> `A`, `14` -> `O`. Only ever called with a column index of this
/// sheet, which is 15 wide.
fn column_letter(index: usize) -> char {
    char::from(b'A' + u8::try_from(index).unwrap_or(0))
}
