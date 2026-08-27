//! Shared helpers for tests that read the real export.

use std::path::PathBuf;

use breadify::sheet::{self, SheetRow};

/// The sample export the specifications were derived from: one delivery day,
/// 2026-03-04, 352 lines over 148 orders.
pub fn sample_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx")
}

/// Every row of the sample, in file order.
pub fn sample_rows() -> Vec<SheetRow> {
    sheet::read(&sample_path()).expect("the sample export should read cleanly")
}
