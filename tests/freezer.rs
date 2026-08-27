//! The loader and the validation pass against the real freezer export.
//!
//! Same sheet shape as the bread export, different warehouse: every figure
//! asserted here was re-derived from the file itself — see
//! `docs/freezer-format.md`.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use breadify::date::ExportKind;
use breadify::route::{self, Route};
use breadify::validate::{self, Severity};
use breadify::{order, sheet};
use support::{freezer_path, freezer_rows};

#[test]
fn reads_every_data_row() {
    assert_eq!(freezer_rows().len(), 231);
}

/// The one place the freezer export breaks a bread-era assumption: `Position`
/// is a warehouse pick slot, and 26 rows have no cell there at all.
#[test]
fn a_missing_position_is_none_not_an_error() {
    let rows = freezer_rows();

    let missing = rows.iter().filter(|row| row.position.is_none()).count();
    assert_eq!(missing, 26, "rows with no Position cell");

    let first = &rows[0];
    assert_eq!(first.excel_row, 2);
    assert_eq!(first.product_id, 9684);
    assert_eq!(first.position, None, "the very first data row has none");
}

/// The slot belongs to the product: no product appears both with and without
/// one, and none appears with two.
#[test]
fn position_is_a_product_attribute() {
    let rows = freezer_rows();

    let mut slots: BTreeMap<u32, BTreeSet<Option<&str>>> = BTreeMap::new();
    for row in &rows {
        slots
            .entry(row.product_id)
            .or_default()
            .insert(row.position.as_deref());
    }

    assert_eq!(slots.len(), 113, "distinct products");
    assert!(slots.values().all(|positions| positions.len() == 1));
    let unplaced = slots.values().filter(|p| p.contains(&None)).count();
    assert_eq!(unplaced, 17, "products with no slot anywhere");
}

/// Seven wholesalers instead of two bakeries, and orders mix them freely.
#[test]
fn the_freezer_warehouse_has_seven_suppliers() {
    let rows = freezer_rows();
    let suppliers: BTreeSet<&str> = rows.iter().map(|row| row.supplier.as_str()).collect();

    assert_eq!(suppliers.len(), 7);
    assert!(suppliers.contains("asko"));
    assert!(suppliers.contains("Sørlandskjøtt AS"));
}

/// Route nicknames now include words alone — and they still group and sort.
#[test]
fn routes_group_in_natural_order() {
    let routes: Vec<Route> = route::group(order::fold(&freezer_rows()));
    let counts: Vec<(String, usize)> = routes
        .iter()
        .map(|route| (route.nickname.clone(), route.line_count()))
        .collect();

    let expected: Vec<(String, usize)> = [
        ("1", 12),
        ("2", 25),
        ("3", 19),
        ("4", 31),
        ("5", 11),
        ("6", 1),
        ("7", 15),
        ("8", 19),
        ("9", 16),
        ("10", 12),
        ("11", 32),
        ("12", 10),
        ("13", 19),
        ("Svg Employee", 1),
        ("hau", 8),
    ]
    .into_iter()
    .map(|(nickname, lines)| (nickname.to_owned(), lines))
    .collect();

    assert_eq!(counts, expected);
}

/// Read as what it is, the freezer sample is as clean as the bread one: the
/// same two notes, and nothing blocking or unfamiliar.
#[test]
fn the_freezer_sample_validates_cleanly_as_a_freezer_export() {
    let findings = validate::run(&freezer_rows(), ExportKind::Freezer);
    let headlines: Vec<&str> = findings
        .iter()
        .map(|finding| finding.headline.as_str())
        .collect();

    assert!(
        findings
            .iter()
            .all(|finding| finding.severity == Severity::Notice),
        "nothing should block or warn: {headlines:?}"
    );
    assert_eq!(
        headlines,
        [
            "28 rows have no position in their route",
            "Column O carries no header",
        ]
    );
}

/// The same rows validated as a *bread* export would drown the reader in
/// notices — which is the point of telling the kinds apart.
#[test]
fn the_freezer_sample_is_full_of_news_to_the_bread_list() {
    let findings = validate::run(&freezer_rows(), ExportKind::Bread);
    let new_suppliers = findings
        .iter()
        .filter(|finding| finding.headline.starts_with("New supplier"))
        .count();

    assert_eq!(new_suppliers, 7);
}

#[test]
fn reading_the_same_file_twice_gives_the_same_rows() {
    let once = sheet::read(&freezer_path()).unwrap();
    let twice = sheet::read(&freezer_path()).unwrap();
    assert_eq!(once, twice);
}
