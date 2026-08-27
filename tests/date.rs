//! Reading the delivery date out of an export's filename.

mod support;

use std::path::Path;

use breadify::date::{self, Date};
use support::sample_path;

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
    ] {
        let outcome = date::from_filename(Path::new(name));
        assert!(outcome.is_err(), "{name} should not parse");
        assert_eq!(outcome.unwrap_err().filename, name);
    }
}

/// The exporter names the freezer list the same way it names the bread one,
/// with a different word in the middle. Matching `PSR-BREAD-` literally left
/// every freezer sheet dateless.
#[test]
fn both_lists_carry_their_dates_and_their_kind() {
    let bread = Path::new("PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx");
    let freezer = Path::new("PSR-FREEZER-2026-01-23-to-2026-01-23 (1).xlsx");

    assert_eq!(
        date::from_filename(bread).unwrap().to_string(),
        "2026-03-04"
    );
    assert_eq!(
        date::from_filename(freezer).unwrap().to_string(),
        "2026-01-23"
    );
    assert_eq!(date::list_word(bread).as_deref(), Some("BREAD"));
    assert_eq!(date::list_word(freezer).as_deref(), Some("FREEZER"));
}

/// The list word is read off the file rather than matched against a list of
/// known ones, and it is taken as everything before the first date — so a
/// hyphenated kind nobody has invented yet would still open.
#[test]
fn an_unknown_list_word_still_carries_dates() {
    let path = Path::new("PSR-DRY-GOODS-2026-05-01-to-2026-05-02.xlsx");

    assert_eq!(date::list_word(path).as_deref(), Some("DRY-GOODS"));
    assert_eq!(
        date::from_filename(path).unwrap().to_string(),
        "2026-05-01 to 2026-05-02"
    );
}

/// A name with a prefix but no list word is not an export.
#[test]
fn a_missing_list_word_is_not_an_export() {
    for name in [
        "PSR-2026-03-04-to-2026-03-04.xlsx",
        "PSR--2026-03-04-to-2026-03-04.xlsx",
        "BREAD-2026-03-04-to-2026-03-04.xlsx",
    ] {
        let path = Path::new(name);
        assert!(
            date::from_filename(path).is_err(),
            "{name} should not parse"
        );
        assert_eq!(date::list_word(path), None, "{name} carries no list word");
    }
}

/// The list word is cut off the stem ten bytes from the end of the dates. A
/// name whose bytes put a multi-byte character across that cut would have
/// panicked inside `split_at`.
#[test]
fn an_oddly_named_file_is_shrugged_at_rather_than_crashed_on() {
    for name in [
        "PSR-€abcdefghi-to-2026-03-04.xlsx",
        "PSR-Ø-to-2026.xlsx",
        "PSR-æøå-2026-03-04-to-2026-03-04.xlsx",
        "PSR-.xlsx",
        "PSR-",
    ] {
        let path = Path::new(name);
        let _ = date::from_filename(path);
        let _ = date::list_word(path);
    }

    // The one of those that is a real name still parses.
    let path = Path::new("PSR-æøå-2026-03-04-to-2026-03-04.xlsx");
    assert_eq!(date::list_word(path).as_deref(), Some("æøå"));
    assert_eq!(date::from_filename(path).unwrap().to_string(), "2026-03-04");
}
