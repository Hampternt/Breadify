//! The validation pass against the real export.
//!
//! The sample file passes every check, which is the point: they exist to catch
//! the day the next export does not.

mod support;

use breadify::date::ExportKind;
use breadify::validate::{self, FindingKind, Severity};
use support::sample_rows;

#[test]
fn the_sample_export_has_no_blocking_findings() {
    let findings = validate::run(&sample_rows(), ExportKind::Bread);
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
fn the_file_reports_what_it_is_as_well_as_what_is_wrong() {
    let findings = validate::run(&sample_rows(), ExportKind::Bread);
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
    let findings = validate::run(&sample_rows(), ExportKind::Bread);
    let notices: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert!(notices.is_empty(), "unexpected notices: {notices:?}");
}

#[test]
fn the_sample_produces_exactly_the_expected_findings() {
    let findings = validate::run(&sample_rows(), ExportKind::Bread);
    assert_eq!(
        findings.len(),
        2,
        "two notes and nothing else: {findings:#?}"
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

    let findings = validate::run(&rows, ExportKind::Bread);
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

    let findings = validate::run(&rows, ExportKind::Bread);
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

    let findings = validate::run(&rows, ExportKind::Bread);
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

    let findings = validate::run(&rows, ExportKind::Bread);
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

    let findings = validate::run(&rows, ExportKind::Bread);
    let notices: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert_eq!(notices, ["New route nickname: north-east"]);
}

/// What is familiar depends on which list it is: a freezer wholesaler on the
/// bread list is news, and the bakeries on the freezer list would be too.
#[test]
fn familiarity_is_per_kind() {
    let mut rows = sample_rows();
    for row in rows.iter_mut().filter(|row| row.supplier == "bakehuset") {
        row.supplier = "asko".to_owned();
    }

    let as_bread: Vec<String> = validate::run(&rows, ExportKind::Bread)
        .into_iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline)
        .collect();
    assert_eq!(as_bread, ["New supplier: asko"]);

    let as_freezer: Vec<String> = validate::run(&rows, ExportKind::Freezer)
        .into_iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline)
        .collect();
    assert_eq!(as_freezer, ["New supplier: sandnes bakeri"]);
}

/// The freezer list names routes with words alone — `hau`, `Svg Employee` —
/// so those are familiar there, and still news on a bread list.
#[test]
fn a_bare_name_route_is_familiar_only_on_the_freezer_list() {
    let mut rows = sample_rows();
    rows[0].route_nickname = "Svg Employee".to_owned();

    let route_notices = |kind| -> Vec<String> {
        validate::run(&rows, kind)
            .into_iter()
            .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
            .filter(|finding| finding.headline.contains("route nickname"))
            .map(|finding| finding.headline)
            .collect()
    };

    assert_eq!(
        route_notices(ExportKind::Bread),
        ["New route nickname: Svg Employee"]
    );
    assert!(route_notices(ExportKind::Freezer).is_empty());
}
