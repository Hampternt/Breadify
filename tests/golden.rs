//! The whole export, checked against an independent extraction of the same
//! file.
//!
//! `data/orders.json` in the design handoff was produced by the design pass
//! from the same workbook, by different code. If this app and that extraction
//! agree row for row, neither is quietly misreading a column.

mod support;

use std::collections::BTreeMap;
use std::path::PathBuf;

use breadify::route::{self, Route};
use breadify::sheet::SheetRow;
use breadify::{order, sheet};
use serde_json::Value;
use support::sample_rows;

fn independent_extraction() -> Vec<Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("Printer page formatting application/design_handoff_breadify/data/orders.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "the design handoff's orders.json should be readable at {}: {error}",
            path.display()
        )
    });
    serde_json::from_str(&text).expect("orders.json is a JSON array")
}

#[test]
fn every_row_matches_the_design_pass_extraction() {
    let ours = sample_rows();
    let theirs = independent_extraction();

    assert_eq!(ours.len(), theirs.len(), "row counts differ");

    for (row, other) in ours.iter().zip(theirs.iter()) {
        // The loader trims text; the design pass kept the file's trailing
        // spaces, of which one comment has one. Trim both sides so the
        // comparison is about the values rather than about that choice.
        let at = |key: &str| other.get(key).and_then(Value::as_str).map(str::trim);
        let context = format!("row {}", row.excel_row);

        assert_eq!(
            Some(row.order_id.to_string().as_str()),
            at("orderId"),
            "{context} order id"
        );
        assert_eq!(
            Some(row.quantity.to_string().as_str()),
            at("qty"),
            "{context} quantity"
        );
        assert_eq!(
            Some(row.product_id.to_string().as_str()),
            at("productId"),
            "{context} product id"
        );
        assert_eq!(
            Some(row.product_name.as_str()),
            at("product"),
            "{context} product name"
        );
        assert_eq!(Some(row.supplier_sku.as_str()), at("sku"), "{context} sku");
        assert_eq!(
            Some(row.position.as_str()),
            at("position"),
            "{context} position"
        );
        assert_eq!(
            Some(row.supplier.as_str()),
            at("supplier"),
            "{context} supplier"
        );
        assert_eq!(
            Some(row.customer.as_str()),
            at("customer"),
            "{context} customer"
        );
        assert_eq!(
            Some(row.delivery_street.as_str()),
            at("street"),
            "{context} street"
        );
        assert_eq!(
            Some(row.route_nickname.as_str()),
            at("route"),
            "{context} route"
        );
        assert_eq!(
            Some(row.route_ordering.to_string().as_str()),
            at("ordering"),
            "{context} ordering"
        );
        assert_eq!(Some(row.region.as_str()), at("region"), "{context} region");
        assert_eq!(
            Some(row.accept_alternatives),
            other.get("alt").and_then(Value::as_bool),
            "{context} accept alternatives"
        );
        assert_eq!(
            row.department.as_deref(),
            at("dept"),
            "{context} department"
        );
        assert_eq!(row.comment.as_deref(), at("comment"), "{context} comment");
    }
}

#[test]
fn each_route_carries_the_lines_it_should() {
    let routes: Vec<Route> = route::group(order::fold(&sample_rows()));
    let counts: Vec<(String, usize)> = routes
        .iter()
        .map(|route| (route.nickname.clone(), route.line_count()))
        .collect();

    let expected: Vec<(String, usize)> = [
        ("1", 24),
        ("2", 13),
        ("3", 30),
        ("4", 22),
        ("5", 34),
        ("6", 29),
        ("7", 17),
        ("8", 13),
        ("9", 25),
        ("10", 20),
        ("11", 29),
        ("12", 14),
        ("13", 26),
        ("14", 32),
        ("hau 1", 12),
        ("hau 2", 12),
    ]
    .into_iter()
    .map(|(nickname, lines)| (nickname.to_owned(), lines))
    .collect();

    assert_eq!(counts, expected);
}

#[test]
fn reading_the_same_file_twice_gives_the_same_rows() {
    let once: Vec<SheetRow> = sheet::read(&support::sample_path()).unwrap();
    let twice: Vec<SheetRow> = sheet::read(&support::sample_path()).unwrap();

    assert_eq!(once, twice);
}

#[test]
fn no_order_is_split_across_routes() {
    let mut routes_of: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for route in route::group(order::fold(&sample_rows())) {
        for stop in route.stops {
            routes_of
                .entry(stop.id)
                .or_default()
                .push(route.nickname.clone());
        }
    }

    assert_eq!(routes_of.len(), 148);
    assert!(routes_of.values().all(|routes| routes.len() == 1));
}
