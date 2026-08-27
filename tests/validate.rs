//! The validation pass against the real export.
//!
//! The sample file is clean apart from two legitimate shared stop sequences,
//! which is the point: the checks exist to catch the day the next export is
//! not.

mod support;

use breadify::validate::{self, FindingKind, Severity};
use support::sample_rows;

#[test]
fn the_sample_export_has_no_blocking_findings() {
    let findings = validate::run(&sample_rows());
    let blocking: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Blocking)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert!(
        blocking.is_empty(),
        "unexpected blocking findings: {blocking:?}"
    );
}

#[test]
fn shared_stop_sequences_are_reported_as_warnings() {
    let findings = validate::run(&sample_rows());
    let shared: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::RepeatedStopSequence)
        .map(|finding| finding.headline.as_str())
        .collect();

    // Route 9 at 1400 is Customer 064's two buildings; route 11 at 1400
    // is Street 17 under three spellings.
    assert_eq!(
        shared,
        [
            "Route 11 has 3 stops at 1400",
            "Route 9 has 2 stops at 1400"
        ]
    );
    assert!(
        findings
            .iter()
            .filter(|finding| finding.kind == FindingKind::RepeatedStopSequence)
            .all(|finding| finding.severity == Severity::Warning)
    );
}

#[test]
fn nothing_in_the_sample_is_unfamiliar() {
    let findings = validate::run(&sample_rows());
    let notices: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert!(notices.is_empty(), "unexpected notices: {notices:?}");
}

#[test]
fn the_sample_produces_exactly_the_two_expected_findings() {
    let findings = validate::run(&sample_rows());
    assert_eq!(findings.len(), 2, "{findings:#?}");
}

#[test]
fn a_disagreeing_order_is_caught() {
    let mut rows = sample_rows();
    let victim = rows
        .iter()
        .position(|row| row.order_id == 1_000_621_240)
        .expect("order 1000621240 is in the sample");
    rows[victim].route_ordering = 999;

    let findings = validate::run(&rows);
    let disagreements: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::OrderLinesDisagree)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert_eq!(
        disagreements,
        ["Order 1000621240 has two values for route ordering"]
    );
}

#[test]
fn an_address_on_two_routes_is_caught() {
    let mut rows = sample_rows();
    let victim = rows
        .iter()
        .position(|row| row.delivery_street == "Street 24")
        .expect("Street 24 is in the sample");
    rows[victim].route_nickname = "hau 1".to_owned();

    let findings = validate::run(&rows);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == FindingKind::AddressOnTwoRoutes
                && finding.headline == "Street 24 is on more than one route")
    );
}

#[test]
fn an_unfamiliar_route_nickname_is_noticed() {
    let mut rows = sample_rows();
    rows[0].route_nickname = "north-east".to_owned();

    let findings = validate::run(&rows);
    let notices: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert_eq!(notices, ["New route nickname: north-east"]);
}
