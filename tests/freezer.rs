//! The loader and the validation pass against the real freezer export.
//!
//! Same sheet shape as the bread export, different warehouse: every figure
//! asserted here was re-derived from the file itself — see
//! `docs/freezer-format.md`.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use breadify::date::ExportKind;
use breadify::layout::{self, Settings};
use breadify::page::{Page, Primitive};
use breadify::route::{self, Route};
use breadify::validate::{self, Severity};
use breadify::{dump, order, sheet};
use support::{freezer_path, freezer_rows};

fn freezer_settings() -> Settings {
    Settings {
        kind: ExportKind::Freezer,
        ..Settings::default()
    }
}

fn freezer_route(nickname: &str) -> Route {
    route::group(order::fold(&freezer_rows()))
        .into_iter()
        .find(|route| route.nickname == nickname)
        .expect("route is in the freezer sample")
}

/// Every text run on every sheet of a route, laid out as a freezer list.
fn freezer_runs(route: &Route) -> Vec<String> {
    layout::paginate(route, None, &freezer_settings(), "PSR-FREEZER-2026-01-23")
        .into_iter()
        .flat_map(|sheet| runs(&sheet.content))
        .collect()
}

fn runs(page: &Page) -> Vec<String> {
    page.primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

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

/// The freezer sheet is the bread sheet minus the picking machinery: every
/// customer and product still prints, no crate glyphs are explained, the page
/// says what it is, and a flat total closes the route (decision F9).
#[test]
fn the_freezer_sheet_is_a_check_list_with_a_flat_total() {
    let route = freezer_route("4");
    let set = freezer_runs(&route);

    for stop in &route.stops {
        assert!(set.contains(&stop.customer), "{} is missing", stop.customer);
        for line in &stop.lines {
            assert!(
                set.contains(&line.product.name),
                "{} is missing",
                line.product.name
            );
        }
    }

    assert!(
        set.iter().any(|run| run.contains("check list")),
        "the page note says what this sheet is"
    );
    assert!(
        !set.iter().any(|run| run == "CRATES"),
        "no crate legend on a checking list"
    );
    assert!(
        set.iter().any(|run| run == "Route 4 total"),
        "the flat total closes the route: {set:?}"
    );
    assert!(
        set.iter().any(|run| run.contains("most to least")),
        "the total says its order"
    );
}

/// The freezer line is checked box, item, dotted note field, missing box —
/// no fixed box anywhere (decision F8).
#[test]
fn the_freezer_line_is_check_note_missing() {
    let set = freezer_runs(&freezer_route("4"));

    assert!(set.iter().any(|run| run == "C"), "the checked box");
    assert!(set.iter().any(|run| run == "M"), "the missing box");
    assert!(!set.iter().any(|run| run == "F"), "no fixed box");
    assert!(
        set.iter()
            .any(|run| run.len() >= 3 && run.chars().all(|character| character == '.')),
        "the dotted note field is set as a leader of full stops"
    );
}

/// The legend reads the freezer line's own words, and its supplier key names
/// the wholesalers this route actually draws from rather than the two
/// bakeries.
#[test]
fn the_freezer_legend_speaks_freezer() {
    let set = freezer_runs(&freezer_route("4"));

    assert!(set.iter().any(|run| run == "Checked"));
    assert!(!set.iter().any(|run| run == "Picked"));
    assert!(!set.iter().any(|run| run == "Fixed"));
    assert!(
        !set.iter().any(|run| run.contains("Sandnes Bakeri")),
        "the bakeries have no place on a freezer sheet"
    );
    assert!(
        set.iter()
            .any(|run| run.contains("Asko") && run.contains(" · ")),
        "the route's own suppliers are the key: {set:?}"
    );
}

/// The longest name on the freezer list fits the flat total's half column, so
/// no row ever runs into its neighbour.
#[test]
fn every_product_name_fits_the_total_column() {
    use breadify::geometry::CONTENT_WIDTH;
    use breadify::layout::metrics::{SIZE_TOTAL_NAME, TOTAL_COLUMN_GAP, TOTAL_QUANTITY_COLUMN};
    use breadify::text::{self, Style};

    let column = (CONTENT_WIDTH - TOTAL_COLUMN_GAP) / 2.0;
    let room = column - (TOTAL_QUANTITY_COLUMN + 2.4);
    let style = Style::new(breadify::font::Face::SpaceGrotesk, SIZE_TOTAL_NAME);

    for row in freezer_rows() {
        let width = text::width(&row.product_name, style);
        assert!(
            width <= room,
            "{:?} is {width:.1} mm against {room:.1} mm of column",
            row.product_name
        );
    }
}

/// The same route through the bread layout still carries its crates and its
/// per-bakery total — the difference is the kind, not the data.
#[test]
fn the_same_route_as_bread_would_carry_crates() {
    let route = freezer_route("4");
    let sheets = layout::paginate(&route, None, &Settings::default(), "PSR-FREEZER-2026-01-23");
    let set: Vec<String> = sheets
        .into_iter()
        .flat_map(|sheet| runs(&sheet.content))
        .collect();

    assert!(set.iter().any(|run| run == "CRATES"));
    assert!(set.iter().any(|run| run == "Route 4 total"));
}

/// The terminal dump follows the page: heading, marker and lines, no crate
/// glyphs, and the flat total at the bottom.
#[test]
fn a_freezer_route_dumps_with_a_flat_total() {
    let text = dump::route(&freezer_route("13"), None, &freezer_settings());

    assert!(text.starts_with("ROUTE 13 — date unknown — 8 stops, 19 lines"));
    assert!(text.contains("Customer 012"));
    assert!(!text.contains('■'), "no crate glyphs: {text}");
    assert!(
        text.contains("Route 13 total — ") && text.contains("most to least"),
        "the flat total closes the dump: {text}"
    );
}
