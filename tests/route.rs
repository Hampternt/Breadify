//! Grouping and sorting the real export into printable routes.
//!
//! A lexical route sort is the single most likely bug in this app, and it
//! fails plausibly — `1, 10, 11, … 2` looks sorted. These tests exist mostly
//! to make that impossible to reintroduce.

mod support;

use breadify::order;
use breadify::route::{self, Route};
use support::sample_rows;

fn sample_routes() -> Vec<Route> {
    route::group(order::fold(&sample_rows()))
}

#[test]
fn routes_sort_naturally_not_lexically() {
    let routes = sample_routes();
    let nicknames: Vec<&str> = routes.iter().map(|route| route.nickname.as_str()).collect();

    assert_eq!(
        nicknames,
        [
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "hau 1",
            "hau 2"
        ]
    );
}

#[test]
fn every_line_survives_grouping() {
    let routes = sample_routes();

    assert_eq!(routes.len(), 16);
    assert_eq!(
        routes.iter().map(Route::line_count).sum::<usize>(),
        352,
        "no line is dropped or duplicated"
    );
    assert_eq!(
        routes.iter().map(|route| route.stops.len()).sum::<usize>(),
        148
    );
}

#[test]
fn route_eight_prints_exactly_as_the_worked_example() {
    let routes = sample_routes();
    let route_eight = routes
        .iter()
        .find(|route| route.nickname == "8")
        .expect("route 8 is in the sample");

    let stops: Vec<(&str, u32)> = route_eight
        .stops
        .iter()
        .map(|stop| (stop.delivery_street.as_str(), stop.sequence))
        .collect();

    assert_eq!(
        stops,
        [
            ("Street 24", 100),
            ("Street 42", 1100),
            ("Street 05", 2100),
            // Unsequenced, tiebroken by address.
            ("Street 55", 0),
            ("Street 71", 0),
        ]
    );
}

#[test]
fn unsequenced_stops_come_last_on_every_route() {
    for route in sample_routes() {
        let first_unsequenced = route.stops.iter().position(|stop| !stop.is_sequenced());
        let Some(first_unsequenced) = first_unsequenced else {
            continue;
        };
        assert!(
            route.stops[first_unsequenced..]
                .iter()
                .all(|stop| !stop.is_sequenced()),
            "route {} interleaves unsequenced stops",
            route.nickname
        );
    }

    let routes = sample_routes();
    let flagged: Vec<&str> = routes
        .iter()
        .filter(|route| route.unsequenced().count() > 0)
        .map(|route| route.nickname.as_str())
        .collect();
    assert_eq!(
        flagged,
        ["2", "4", "5", "8", "9", "11", "12", "hau 1", "hau 2"],
        "routes needing the unsequenced flag"
    );
}

#[test]
fn stops_sharing_a_sequence_break_the_tie_by_address_then_department() {
    let routes = sample_routes();

    let route_nine = routes.iter().find(|r| r.nickname == "9").unwrap();
    let tied: Vec<&str> = route_nine
        .stops
        .iter()
        .filter(|stop| stop.sequence == 1400)
        .map(|stop| stop.delivery_street.as_str())
        .collect();
    assert_eq!(tied, ["Street 64", "Street 65"]);

    let route_eleven = routes.iter().find(|r| r.nickname == "11").unwrap();
    let tied: Vec<(&str, Option<&str>)> = route_eleven
        .stops
        .iter()
        .filter(|stop| stop.sequence == 1400)
        .map(|stop| (stop.delivery_street.as_str(), stop.department.as_deref()))
        .collect();
    // Re-derived after the samples were anonymised: the placeholders sort
    // differently from the names they replaced, and this list is the sort
    // itself — address first, then department, with no department first.
    assert_eq!(
        tied,
        [
            ("Street 112", Some("Department 32")),
            ("Street 17", None),
            ("Street 17", None),
            ("Street 17", None),
            ("Street 17", Some("Department 09")),
            ("Street 62", Some("Department 16")),
            ("Street 62", Some("Department 22")),
            ("Street 62", Some("Department 22")),
        ]
    );
}

#[test]
fn sorting_is_stable_across_runs() {
    let stop_ids = || -> Vec<i64> {
        sample_routes()
            .into_iter()
            .flat_map(|route| route.stops.into_iter().map(|stop| stop.id))
            .collect()
    };

    let first = stop_ids();
    let second = stop_ids();

    assert_eq!(first, second);
}

#[test]
fn nicknames_this_export_has_never_seen_still_sort_sensibly() {
    use breadify::route::{RouteKey, natural_key};

    assert_eq!(natural_key("7"), RouteKey::Numbered(7, ""));
    assert_eq!(natural_key("hau 10"), RouteKey::Named("hau", 10));
    assert_eq!(natural_key("nord"), RouteKey::Named("nord", 0));
    assert!(natural_key("14") < natural_key("hau 1"));
    assert!(natural_key("hau 2") < natural_key("hau 10"));
    assert!(natural_key("2") < natural_key("10"));
}
