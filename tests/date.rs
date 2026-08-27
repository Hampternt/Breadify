//! Reading the delivery date — and which list it is — out of an export's
//! filename.

mod support;

use std::path::Path;

use breadify::date::{self, Date, ExportKind};
use support::{freezer_path, sample_path};

#[test]
fn the_sample_is_a_single_day_despite_its_download_suffix() {
    let dates = date::from_filename(&sample_path()).expect("the sample is named by the exporter");

    assert_eq!(
        dates.from,
        Date {
            year: 2026,
            month: 3,
            day: 4
        }
    );
    assert!(dates.is_single_day());
    assert_eq!(dates.to_string(), "2026-03-04");
}

#[test]
fn a_name_without_a_download_suffix_reads_the_same() {
    let dates = date::from_filename(Path::new("PSR-BREAD-2026-03-04-to-2026-03-04.xlsx"))
        .expect("the plain name parses");

    assert_eq!(dates.to_string(), "2026-03-04");
}

#[test]
fn a_span_of_days_keeps_both_ends() {
    let dates = date::from_filename(Path::new("PSR-BREAD-2026-03-04-to-2026-03-06.xlsx"))
        .expect("a span parses");

    assert!(!dates.is_single_day());
    assert_eq!(dates.to_string(), "2026-03-04 to 2026-03-06");
}

#[test]
fn an_unrecognisable_name_asks_rather_than_guesses() {
    for name in [
        "orders.xlsx",
        "PSR-BREAD-2026-03-04.xlsx",
        "PSR-BREAD-2026-13-04-to-2026-03-06.xlsx",
        "PSR-BREAD-not-a-date-to-2026-03-06.xlsx",
        "PSR-FREEZER-2026-01-23.xlsx",
    ] {
        let outcome = date::from_filename(Path::new(name));
        assert!(outcome.is_err(), "{name} should not parse");
        assert_eq!(outcome.unwrap_err().filename, name);
    }
}

#[test]
fn a_freezer_export_carries_its_dates_the_same_way() {
    let dates =
        date::from_filename(&freezer_path()).expect("the freezer sample is named by the exporter");

    assert_eq!(
        dates.from,
        Date {
            year: 2026,
            month: 1,
            day: 23
        }
    );
    assert!(dates.is_single_day());
}

#[test]
fn the_filename_says_which_list_it_is() {
    assert_eq!(date::export_kind(&sample_path()), Some(ExportKind::Bread));
    assert_eq!(
        date::export_kind(&freezer_path()),
        Some(ExportKind::Freezer)
    );
    assert_eq!(date::export_kind(Path::new("orders.xlsx")), None);
}
