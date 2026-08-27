//! The `dump` output, which is how pack 1 is read by eye.

mod support;

use breadify::layout::Settings;
use breadify::list::Kind;
use breadify::route::Route;
use breadify::{date, dump, order, route};
use support::{sample_path, sample_rows};

fn route_eight() -> Route {
    route::group(order::fold(&sample_rows()))
        .into_iter()
        .find(|route| route.nickname == "8")
        .expect("route 8 is in the sample")
}

#[test]
fn a_route_dumps_in_the_shape_of_the_worked_example() {
    let dates = date::from_filename(&sample_path()).ok();
    let text = dump::route(&route_eight(), dates, &Settings::default());

    assert!(text.starts_with("ROUTE 8 — 2026-03-04 — 5 stops, 13 lines"));

    let flag = text
        .find("no position assigned")
        .expect("the unsequenced tail is flagged");
    assert!(
        text.find("Customer 054").is_some_and(|stop| stop > flag),
        "the flag comes before the stops it covers"
    );
    assert!(
        text.find("Customer 005")
            .is_some_and(|stop| stop < flag),
        "sequenced stops come before the flag"
    );

    assert!(
        text.contains("WANT SUBSTITUTE: FALSE"),
        "A3 refuses substitutes"
    );
    assert!(text.contains("BH  Barnehagebrødet - Oppskåret 750g"));
    assert!(text.contains("Route 8 total — 7 types · 43 units"));
    assert!(
        text.contains("10  Barnehagebrødet - Oppskåret 750g  ●"),
        "the route's one full tray shows a dot"
    );
}

#[test]
fn a_route_with_no_unsequenced_stops_shows_no_flag() {
    let route_one = route::group(order::fold(&sample_rows()))
        .into_iter()
        .find(|route| route.nickname == "1")
        .expect("route 1 is in the sample");

    let text = dump::route(&route_one, None, &Settings::default());
    assert!(!text.contains("no position assigned"));
    assert!(text.starts_with("ROUTE 1 — date unknown — 11 stops, 24 lines"));
}

/// The terminal dump reads the same model the sheet is drawn from, so a
/// freezer route has no crate glyphs there either.
#[test]
fn a_freezer_route_dumps_without_crates() {
    let route = route::group(order::fold(&support::freezer_rows()))
        .into_iter()
        .find(|route| route.nickname == "11")
        .expect("freezer route 11");

    let freezer = Settings::default().for_list(Kind::Freezer);
    let text = dump::route(&route, None, &freezer);
    assert!(!text.contains('■') && !text.contains('◪'), "{text}");

    // The same route read as bread does draw them, so the assertion above is
    // about the list and not about the route being small.
    let bread = dump::route(&route, None, &Settings::default());
    assert!(bread.contains('■') || bread.contains('◪'));
}
