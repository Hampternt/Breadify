//! Checking a file that read cleanly for the things that would make its
//! printed pages wrong.
//!
//! Every invariant the specification relies on holds across the one export it
//! was derived from — one file, one day. That is not proof, so the loader
//! checks rather than assumes, and reports what it finds instead of refusing
//! the file. See `docs/excel-format.md` §6.

use std::collections::{BTreeMap, BTreeSet};

use crate::date::ExportKind;
use crate::sheet::SheetRow;

/// A column name paired with a way of reading that column off a row, as text.
/// Checks that treat several columns the same way iterate over these.
type TextColumn = (&'static str, fn(&SheetRow) -> String);

/// The same, for columns read by reference because they are always present.
type RequiredColumn = (&'static str, fn(&SheetRow) -> &str);

/// How much a finding should worry the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The printed pages would be wrong. Do not print until it is resolved.
    Blocking,
    /// Something to look at, but the pages are still usable.
    Warning,
    /// The file contains something this app has not seen before.
    Notice,
}

/// Which check produced a finding. The text is for the reader; this is for
/// grouping, filtering and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingKind {
    /// A required cell holds an empty string.
    BlankRequiredField,
    /// Lines of one order disagree about something that belongs to the order.
    OrderLinesDisagree,
    /// One delivery address appears on more than one route.
    AddressOnTwoRoutes,
    /// One product identifier carries more than one name, SKU or supplier.
    ProductDetailsDisagree,
    /// A value this app has never seen in that column before.
    UnfamiliarValue,
    /// Stops the export gave no position in their route.
    UnsequencedStops,
    /// The fifteenth column, which carries data but no header.
    UnlabelledColumn,
}

/// Something worth telling the user about the file they opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub kind: FindingKind,
    /// One line, for the findings list.
    pub headline: String,
    /// The specifics, for when they open it.
    pub detail: String,
    /// Worksheet rows involved, so the user can go and look.
    pub rows: Vec<usize>,
}

/// Runs every check over a file's rows.
///
/// The invariants are the same for both lists; what counts as *familiar* —
/// suppliers, route nickname shapes — depends on which list this is, so the
/// caller says. A file whose name reveals nothing validates as bread, the
/// kind every check was first derived from.
///
/// Returns the findings most severe first; an empty list means the file
/// matches every invariant the specification relies on.
pub fn run(rows: &[SheetRow], kind: ExportKind) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(blank_required_fields(rows));
    findings.extend(orders_that_disagree(rows));
    findings.extend(addresses_on_two_routes(rows));
    findings.extend(products_that_disagree(rows));
    findings.extend(unfamiliar_values(rows, kind));
    findings.extend(unsequenced_stops(rows));
    findings.extend(unlabelled_column(rows));
    findings.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then(left.kind.cmp(&right.kind))
            .then(left.rows.cmp(&right.rows))
    });
    findings
}

/// Not a problem — but the user should know before printing that some stops
/// will come out after the sequenced ones, under a flag, because the export
/// gave them no position.
fn unsequenced_stops(rows: &[SheetRow]) -> Option<Finding> {
    let unsequenced: Vec<&SheetRow> = rows.iter().filter(|row| row.route_ordering == 0).collect();
    if unsequenced.is_empty() {
        return None;
    }

    let customers: BTreeSet<&str> = unsequenced
        .iter()
        .map(|row| row.customer.as_str())
        .collect();
    let routes: BTreeSet<&str> = unsequenced
        .iter()
        .map(|row| row.route_nickname.as_str())
        .collect();

    Some(Finding {
        severity: Severity::Notice,
        kind: FindingKind::UnsequencedStops,
        headline: format!("{} rows have no position in their route", unsequenced.len()),
        detail: format!(
            "{} customers across {} routes. They print after the sequenced stops, \
             under a flag saying the order is the driver's to choose.",
            customers.len(),
            routes.len()
        ),
        rows: unsequenced.iter().map(|row| row.excel_row).collect(),
    })
}

/// The fifteenth column has data in every row and no header. Worth saying
/// once, because a reader that trusts the header row drops it silently.
fn unlabelled_column(rows: &[SheetRow]) -> Option<Finding> {
    let values: BTreeSet<&str> = rows.iter().map(|row| row.region.as_str()).collect();
    if values.is_empty() {
        return None;
    }

    Some(Finding {
        severity: Severity::Notice,
        kind: FindingKind::UnlabelledColumn,
        headline: "Column O carries no header".to_owned(),
        detail: format!(
            "Read positionally. It holds {} on all {} rows; nothing is keyed off it.",
            quoted(&values.iter().map(|value| (*value).to_owned()).collect()),
            rows.len()
        ),
        rows: Vec::new(),
    })
}

/// The columns that must carry text. A cell can exist and still be blank,
/// which the reader cannot catch on its own.
fn blank_required_fields(rows: &[SheetRow]) -> Vec<Finding> {
    let required: [RequiredColumn; 6] = [
        ("Product Name", |row| &row.product_name),
        ("Supplier SKU", |row| &row.supplier_sku),
        ("Supplier", |row| &row.supplier),
        ("Customer", |row| &row.customer),
        ("Delivery street", |row| &row.delivery_street),
        ("Route nickname", |row| &row.route_nickname),
    ];

    rows.iter()
        .flat_map(|row| {
            required.iter().filter_map(move |(column, read)| {
                if !read(row).is_empty() {
                    return None;
                }
                Some(Finding {
                    severity: Severity::Blocking,
                    kind: FindingKind::BlankRequiredField,
                    headline: format!("{column} is empty on row {}", row.excel_row),
                    detail: format!(
                        "Row {} of order {} has no {column}. Every line needs one.",
                        row.excel_row, row.order_id
                    ),
                    rows: vec![row.excel_row],
                })
            })
        })
        .collect()
}

/// Everything except the product and the quantity belongs to the order and is
/// repeated onto each of its lines. If two lines disagree, folding them into
/// one order would silently pick a winner.
fn orders_that_disagree(rows: &[SheetRow]) -> Vec<Finding> {
    let attributes: [TextColumn; 6] = [
        ("customer", |row| row.customer.clone()),
        ("department", |row| {
            row.department.clone().unwrap_or_default()
        }),
        ("delivery street", |row| row.delivery_street.clone()),
        ("route", |row| row.route_nickname.clone()),
        ("route ordering", |row| row.route_ordering.to_string()),
        ("accept alternatives", |row| {
            row.accept_alternatives.to_string()
        }),
    ];

    group_by(rows, |row| row.order_id)
        .into_iter()
        .flat_map(|(order_id, lines)| {
            let mut findings: Vec<Finding> = attributes
                .iter()
                .filter_map(|(name, read)| {
                    let values: BTreeSet<String> = lines.iter().map(|row| read(row)).collect();
                    if values.len() < 2 {
                        return None;
                    }
                    Some(Finding {
                        severity: Severity::Blocking,
                        kind: FindingKind::OrderLinesDisagree,
                        headline: format!("Order {order_id} has two values for {name}"),
                        detail: format!(
                            "The lines of order {order_id} disagree about {name}: {}. \
                             Everything but the product and the quantity belongs to the order.",
                            quoted(&values)
                        ),
                        rows: lines.iter().map(|row| row.excel_row).collect(),
                    })
                })
                .collect();

            findings.extend(two_notes_on_one_order(order_id, &lines));
            findings
        })
        .collect()
}

/// The comment is the one order-level value a line may simply not carry: this
/// export repeats it onto every line, but writing it once would be just as
/// valid, and [`crate::order::fold`] takes whichever line has it. Only two
/// *different* notes on one order are a problem, because then there is no
/// saying which one the customer meant.
fn two_notes_on_one_order(order_id: i64, lines: &[&SheetRow]) -> Option<Finding> {
    let notes: BTreeSet<&str> = lines
        .iter()
        .filter_map(|row| row.comment.as_deref())
        .collect();
    if notes.len() < 2 {
        return None;
    }

    Some(Finding {
        severity: Severity::Blocking,
        kind: FindingKind::OrderLinesDisagree,
        headline: format!("Order {order_id} carries two different notes"),
        detail: format!(
            "The lines of order {order_id} carry {}. Only one can be printed.",
            quoted(&notes.iter().map(|note| (*note).to_owned()).collect())
        ),
        rows: lines.iter().map(|row| row.excel_row).collect(),
    })
}

/// The address is the most reliable identity a stop has, and the printed order
/// of a route depends on it belonging to exactly one route.
fn addresses_on_two_routes(rows: &[SheetRow]) -> Vec<Finding> {
    group_by(rows, |row| row.delivery_street.clone())
        .into_iter()
        .filter_map(|(address, lines)| {
            let routes: BTreeSet<&str> = lines
                .iter()
                .map(|row| row.route_nickname.as_str())
                .collect();
            if routes.len() < 2 {
                return None;
            }
            Some(Finding {
                severity: Severity::Blocking,
                kind: FindingKind::AddressOnTwoRoutes,
                headline: format!("{address} is on more than one route"),
                detail: format!(
                    "{address} appears on routes {}. One address belongs to one route.",
                    routes.iter().copied().collect::<Vec<_>>().join(", ")
                ),
                rows: lines.iter().map(|row| row.excel_row).collect(),
            })
        })
        .collect()
}

/// A product identifier that means two different things would put the wrong
/// bread on a page.
fn products_that_disagree(rows: &[SheetRow]) -> Vec<Finding> {
    let attributes: [TextColumn; 3] = [
        ("name", |row| row.product_name.clone()),
        ("SKU", |row| row.supplier_sku.clone()),
        ("supplier", |row| row.supplier.clone()),
    ];

    group_by(rows, |row| row.product_id)
        .into_iter()
        .flat_map(|(product_id, lines)| {
            attributes
                .iter()
                .filter_map(move |(name, read)| {
                    let values: BTreeSet<String> = lines.iter().map(|row| read(row)).collect();
                    if values.len() < 2 {
                        return None;
                    }
                    Some(Finding {
                        severity: Severity::Blocking,
                        kind: FindingKind::ProductDetailsDisagree,
                        headline: format!("Product {product_id} has two values for {name}"),
                        detail: format!(
                            "Product {product_id} appears with {} as its {name}.",
                            quoted(&values)
                        ),
                        rows: lines.iter().map(|row| row.excel_row).collect(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The suppliers every export of a kind so far has drawn from, in the
/// spelling the file uses, compared case-insensitively. Bread has its two
/// bakeries; the freezer list draws on a warehouse of wholesalers.
fn known_suppliers(kind: ExportKind) -> &'static [&'static str] {
    match kind {
        ExportKind::Bread => &["sandnes bakeri", "bakehuset"],
        ExportKind::Freezer => &[
            "asko",
            "fatland",
            "møremat",
            "ytterøy",
            "holmens as",
            "sørlandskjøtt as",
            "gabbas",
        ],
    }
}

/// Values outside what every export of this kind so far has contained. Not
/// wrong — the app has simply never seen them, and someone should look before
/// printing.
fn unfamiliar_values(rows: &[SheetRow], kind: ExportKind) -> Vec<Finding> {
    let mut findings = Vec::new();

    findings.extend(unfamiliar(rows, "region", |row| {
        (row.region != "Stavanger").then(|| row.region.clone())
    }));
    findings.extend(unfamiliar(rows, "supplier", |row| {
        let known = known_suppliers(kind)
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&row.supplier));
        (!known).then(|| row.supplier.clone())
    }));
    findings.extend(unfamiliar(rows, "route nickname", |row| {
        (!is_familiar_route(&row.route_nickname, kind)).then(|| row.route_nickname.clone())
    }));

    findings
}

/// Collects one finding per unfamiliar value, naming every row it appears on.
fn unfamiliar(
    rows: &[SheetRow],
    column: &str,
    odd_one_out: impl Fn(&SheetRow) -> Option<String>,
) -> Vec<Finding> {
    let mut by_value: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for row in rows {
        if let Some(value) = odd_one_out(row) {
            by_value.entry(value).or_default().push(row.excel_row);
        }
    }

    by_value
        .into_iter()
        .map(|(value, rows)| Finding {
            severity: Severity::Notice,
            kind: FindingKind::UnfamiliarValue,
            headline: format!("New {column}: {value}"),
            detail: format!(
                "{value} has not appeared in this column before. It appears on {} row(s).",
                rows.len()
            ),
            rows,
        })
        .collect()
}

/// A route nickname is familiar if it is a number, or a name followed by one —
/// `7`, `hau 2`. The freezer list also names routes with words alone — `hau`,
/// `Svg Employee` — so a digit-free name is familiar there too. Everything
/// here sorts naturally; anything else needs a human.
fn is_familiar_route(nickname: &str, kind: ExportKind) -> bool {
    if nickname.is_empty() {
        return false;
    }
    if nickname.chars().all(|character| character.is_ascii_digit()) {
        return true;
    }
    if kind == ExportKind::Freezer && !nickname.chars().any(|character| character.is_ascii_digit())
    {
        return true;
    }
    let Some((name, number)) = nickname.rsplit_once(' ') else {
        return false;
    };
    !name.is_empty()
        && !number.is_empty()
        && number.chars().all(|character| character.is_ascii_digit())
}

/// Groups rows by a key, keeping both the keys and the rows within each key in
/// a stable order so findings read the same on every run.
fn group_by<Key: Ord, Read: Fn(&SheetRow) -> Key>(
    rows: &[SheetRow],
    key: Read,
) -> BTreeMap<Key, Vec<&SheetRow>> {
    let mut grouped: BTreeMap<Key, Vec<&SheetRow>> = BTreeMap::new();
    for row in rows {
        grouped.entry(key(row)).or_default().push(row);
    }
    grouped
}

/// `a, b and c`, quoted, for a sentence.
fn quoted(values: &BTreeSet<String>) -> String {
    let quoted: Vec<String> = values.iter().map(|value| format!("{value:?}")).collect();
    match quoted.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}
