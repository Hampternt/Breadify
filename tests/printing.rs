//! Choosing what to print.

mod support;

use std::collections::BTreeSet;

use breadify::app::print;
use breadify::layout::Settings;
use breadify::route::{self, Route};
use breadify::{order, pdf};
use support::sample_rows;

fn routes() -> Vec<Route> {
    route::group(order::fold(&sample_rows()))
}

#[test]
fn everything_means_every_route() {
    let routes = routes();
    let all = print::everything(&routes);

    assert_eq!(all.len(), 16);
    let sheets = print::day_for(&routes, &all, None, &Settings::default(), "PSR-BREAD");
    assert_eq!(sheets.len(), 27);
}

#[test]
fn selecting_one_route_prints_only_that_route() {
    let routes = routes();
    let one: BTreeSet<String> = ["5".to_owned()].into_iter().collect();
    let sheets = print::day_for(&routes, &one, None, &Settings::default(), "PSR-BREAD");

    assert_eq!(sheets.len(), 2, "route 5 takes two sheets");
    assert!(sheets.iter().all(|sheet| sheet.route == "5"));
}

#[test]
fn selecting_nothing_prints_nothing() {
    let sheets = print::day_for(
        &routes(),
        &BTreeSet::new(),
        None,
        &Settings::default(),
        "PSR-BREAD",
    );
    assert!(sheets.is_empty());
}

#[test]
fn the_selection_renders_to_a_pdf_in_printing_order() {
    let routes = routes();
    let chosen: BTreeSet<String> = ["8".to_owned(), "hau 1".to_owned()].into_iter().collect();
    let sheets = print::day_for(&routes, &chosen, None, &Settings::default(), "PSR-BREAD");

    let order: Vec<&str> = sheets.iter().map(|sheet| sheet.route.as_str()).collect();
    assert_eq!(
        order,
        ["8", "hau 1"],
        "numbered routes come before named ones"
    );

    let pages: Vec<_> = sheets.into_iter().map(|sheet| sheet.content).collect();
    let bytes = pdf::render(&pages, "Breadify pick lists").expect("renders");
    assert!(bytes.starts_with(b"%PDF"));
}
