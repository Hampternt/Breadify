//! Route totals against the real export.
//!
//! Every figure was recomputed from the file. The ten-dots are the trap: they
//! count full tens *within an order*, so route 11's Kneippbrød (quantities 7,
//! 4, 10, 2, 8, 6, 11, 20 — 68 units) shows four dots, not six.

mod support;

use breadify::route::{self, Route};
use breadify::{order, total};
use support::sample_rows;

fn sample_routes() -> Vec<Route> {
    route::group(order::fold(&sample_rows()))
}

fn route_named(routes: &[Route], nickname: &str) -> Route {
    routes
        .iter()
        .find(|route| route.nickname == nickname)
        .expect("route is in the sample")
        .clone()
}

#[test]
fn route_eight_totals_exactly() {
    let routes = sample_routes();
    let total = total::of(&route_named(&routes, "8"));

    assert_eq!(total.types(), 7);
    assert_eq!(total.units(), 43);
    assert_eq!(total.full_tens(), 1);

    let columns: Vec<(&str, usize, u32)> = total
        .columns
        .iter()
        .map(|column| (column.supplier.as_str(), column.types(), column.units()))
        .collect();
    assert_eq!(
        columns,
        [("sandnes bakeri", 6, 33), ("bakehuset", 1, 10)],
        "Sandnes Bakeri leads, whatever the volumes"
    );

    let sandnes: Vec<(&str, u32, u32)> = total.columns[0]
        .lines
        .iter()
        .map(|line| (line.product.name.as_str(), line.units, line.full_tens))
        .collect();
    assert_eq!(
        sandnes,
        [
            ("Grovbrød M/sirup Oppdelt Sandnes Bakeri", 10, 0),
            ("Ryfylkebrød Sandnes Oppdelt Bakeri", 8, 0),
            // Two at five: the name breaks the tie.
            ("Sandnesbrød Oppdelt Sandnes Bakeri", 5, 0),
            ("Sekskornsbrød Oppdelt Sandnes Bakeri", 5, 0),
            ("Havrebrød Oppdelt Sandnes Bakeri", 4, 0),
            ("Dansk Rugbrød Hel Sandnes Bakeri", 1, 0),
        ]
    );

    let bakehuset = &total.columns[1].lines;
    assert_eq!(bakehuset.len(), 1);
    assert_eq!(
        bakehuset[0].product.name,
        "Barnehagebrødet - Oppskåret 750g"
    );
    assert_eq!(bakehuset[0].full_tens, 1, "one order of ten pulls a tray");
}

#[test]
fn ten_dots_count_full_tens_within_an_order() {
    let routes = sample_routes();
    let total = total::of(&route_named(&routes, "11"));

    let kneipp = total.columns[0]
        .lines
        .iter()
        .find(|line| line.product.name.starts_with("Kneippbrød"))
        .expect("route 11 carries Kneippbrød");

    assert_eq!(kneipp.units, 68);
    assert_eq!(kneipp.full_tens, 4, "not 6, which is 68 / 10");
}

#[test]
fn the_prototype_routes_match_their_recomputed_totals() {
    let routes = sample_routes();

    // route, types, units, full tens, (sandnes types, units), (bakehuset types, units)
    let expected = [
        ("5", 14, 117, 4, (10, 85), (4, 32)),
        ("8", 7, 43, 1, (6, 33), (1, 10)),
        ("11", 14, 172, 6, (10, 139), (4, 33)),
        ("14", 13, 102, 2, (7, 62), (6, 40)),
    ];

    for (nickname, types, units, tens, sandnes, bakehuset) in expected {
        let total = total::of(&route_named(&routes, nickname));
        assert_eq!(total.types(), types, "route {nickname} types");
        assert_eq!(total.units(), units, "route {nickname} units");
        assert_eq!(total.full_tens(), tens, "route {nickname} ten-dots");
        assert_eq!(
            (total.columns[0].types(), total.columns[0].units()),
            sandnes,
            "route {nickname} Sandnes column"
        );
        assert_eq!(
            (total.columns[1].types(), total.columns[1].units()),
            bakehuset,
            "route {nickname} Bakehuset column"
        );
    }
}

#[test]
fn every_bread_in_the_export_is_counted_exactly_once() {
    let routes = sample_routes();
    let counted: u32 = routes.iter().map(|route| total::of(route).units()).sum();

    assert_eq!(counted, 1581, "every quantity in the file");
}

#[test]
fn the_summary_line_pluralises() {
    assert_eq!(total::summary(1, 10), "1 type · 10 units");
    assert_eq!(total::summary(6, 33), "6 types · 33 units");
    assert_eq!(total::summary(1, 1), "1 type · 1 unit");
}
