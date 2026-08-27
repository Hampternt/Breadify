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
            "Route 11 has 3 addresses at 1400",
            "Route 9 has 2 addresses at 1400"
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
fn the_file_reports_what_it_is_as_well_as_what_is_wrong() {
    let findings = validate::run(&sample_rows());
    let notes: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Notice)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert_eq!(
        notes,
        [
            "37 rows have no position in their route",
            "Column O carries no header",
        ]
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
fn the_sample_produces_exactly_the_expected_findings() {
    let findings = validate::run(&sample_rows());
    assert_eq!(
        findings.len(),
        4,
        "two warnings and two notes: {findings:#?}"
    );
}

#[test]
fn a_note_written_on_only_one_line_of_an_order_is_fine() {
    let mut rows = sample_rows();
    let mut kept = false;
    for row in rows.iter_mut().filter(|row| row.order_id == 1_000_621_028) {
        if row.comment.is_some() && !kept {
            kept = true;
            continue;
        }
        row.comment = None;
    }

    let findings = validate::run(&rows);
    let blocking: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Blocking)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert!(
        blocking.is_empty(),
        "folding already handles this: {blocking:?}"
    );
}

#[test]
fn two_different_notes_on_one_order_are_caught() {
    let mut rows = sample_rows();
    let victim = rows
        .iter()
        .position(|row| row.order_id == 1_000_621_028)
        .expect("order 1000621028 is in the sample");
    rows[victim].comment = Some("Something else entirely".to_owned());

    let findings = validate::run(&rows);
    let headlines: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::OrderLinesDisagree)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert_eq!(headlines, ["Order 1000621028 carries two different notes"]);
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
