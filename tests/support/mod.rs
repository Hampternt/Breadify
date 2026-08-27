//! Shared helpers for tests that read the real export.
//!
//! Every test binary compiles this module separately, and none of them uses
//! all of it, so unused helpers are expected here rather than a smell.
#![allow(dead_code)]

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

/// The freezer sample: one delivery day, 2026-01-23, 231 lines over 115
/// orders, from seven suppliers rather than two bakeries.
pub fn freezer_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("PSR-FREEZER-2026-01-23-to-2026-01-23 (1).xlsx")
}

/// Every row of the freezer sample, in file order.
pub fn freezer_rows() -> Vec<SheetRow> {
    sheet::read(&freezer_path()).expect("the freezer sample should read cleanly")
}
