//! The validation pass against the real export.
//!
//! The sample file passes every check, which is the point: they exist to catch
//! the day the next export does not.

mod support;

use breadify::list::Kind;
use breadify::validate::{self, FindingKind, Severity};
use support::{freezer_rows, sample_rows};

#[test]
fn the_sample_export_has_no_blocking_findings() {
    let findings = validate::run(&sample_rows(), &Kind::Bread);
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
    let findings = validate::run(&sample_rows(), &Kind::Bread);
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
    let findings = validate::run(&sample_rows(), &Kind::Bread);
    let notices: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline.as_str())
        .collect();

    assert!(notices.is_empty(), "unexpected notices: {notices:?}");
}

#[test]
fn the_sample_produces_exactly_the_expected_findings() {
    let findings = validate::run(&sample_rows(), &Kind::Bread);
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

    let findings = validate::run(&rows, &Kind::Bread);
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

    let findings = validate::run(&rows, &Kind::Bread);
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

    let findings = validate::run(&rows, &Kind::Bread);
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

    let findings = validate::run(&rows, &Kind::Bread);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == FindingKind::AddressOnTwoRoutes
                && finding.headline == "Street 24 is on more than one route")
    );
}

/// The check exists because a nickname the sorter cannot place would print in
/// an order nobody chose. A nickname it *can* place is not a finding, whatever
/// it is called.
#[test]
fn a_route_nickname_the_sorter_cannot_place_is_noticed() {
    let mut rows = sample_rows();
    rows[0].route_nickname = "   ".to_owned();

    let findings = validate::run(&rows, &Kind::Bread);
    assert_eq!(unfamiliar(&findings), ["New route nickname:    "]);

    for placeable in ["north-east", "hau", "Svg Employee", "14", "hau 2"] {
        let mut rows = sample_rows();
        rows[0].route_nickname = placeable.to_owned();
        let findings = validate::run(&rows, &Kind::Bread);
        assert!(
            unfamiliar(&findings).is_empty(),
            "{placeable:?} sorts fine and should not be a finding"
        );
    }
}

/// The freezer list buys from seven suppliers on routes called `hau` and `Svg
/// Employee`. Every one of those raised a notice when the familiar values were
/// the bread list's — nine findings on a clean file, which teaches the user to
/// scroll past the step rather than read it.
#[test]
fn a_clean_freezer_export_raises_no_unfamiliar_values() {
    let findings = validate::run(&freezer_rows(), &Kind::Freezer);
    assert_eq!(unfamiliar(&findings), Vec::<&str>::new());
}

/// And the same file read as bread — a renamed export — says all nine, which
/// is what makes the test above mean something.
#[test]
fn the_freezer_export_read_as_bread_says_so_loudly() {
    let findings = validate::run(&freezer_rows(), &Kind::Bread);
    let notices = unfamiliar(&findings);
    assert_eq!(notices.len(), 7, "seven suppliers no bakery list knows");
    assert!(notices.iter().any(|headline| headline.contains("asko")));
}

/// The headlines of every unfamiliar-value notice, in the order they are
/// reported.
fn unfamiliar(findings: &[breadify::validate::Finding]) -> Vec<&str> {
    findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::UnfamiliarValue)
        .map(|finding| finding.headline.as_str())
        .collect()
}
