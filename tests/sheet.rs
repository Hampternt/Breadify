//! The loader against the real export.
//!
//! Every figure asserted here was re-derived from the file itself, not copied
//! from a document — see `docs/excel-format.md` §2.

mod support;

use support::sample_rows;

#[test]
fn reads_every_data_row() {
    assert_eq!(sample_rows().len(), 352);
}

#[test]
fn absent_cells_are_none_not_blank() {
    let rows = sample_rows();

    let with_department = rows.iter().filter(|row| row.department.is_some()).count();
    let with_comment = rows.iter().filter(|row| row.comment.is_some()).count();
    let with_neither = rows
        .iter()
        .filter(|row| row.department.is_none() && row.comment.is_none())
        .count();

    assert_eq!(with_department, 91, "rows carrying a department");
    assert_eq!(with_comment, 10, "rows carrying a comment");
    assert_eq!(with_neither, 251, "rows carrying neither");

    // No row in this export carries both, but that is a coincidence of one
    // file rather than a rule, so nothing asserts it.
    assert_eq!(with_department + with_comment + with_neither, rows.len());
}

#[test]
fn order_ids_survive_being_stored_as_floats() {
    let rows = sample_rows();
    let ids: Vec<i64> = rows.iter().map(|row| row.order_id).collect();

    assert_eq!(ids.iter().min(), Some(&1_000_617_801));
    assert_eq!(ids.iter().max(), Some(&1_000_622_767));
}

#[test]
fn accept_alternatives_is_a_boolean() {
    let rows = sample_rows();
    let refusing = rows.iter().filter(|row| !row.accept_alternatives).count();

    assert_eq!(refusing, 34, "lines whose order refuses substitutes");
    assert_eq!(rows.len() - refusing, 318);
}

#[test]
fn numeric_looking_columns_that_are_really_text_stay_text() {
    let rows = sample_rows();

    let nicknames: std::collections::BTreeSet<&str> =
        rows.iter().map(|row| row.route_nickname.as_str()).collect();
    assert_eq!(nicknames.len(), 16);
    assert!(nicknames.contains("hau 1"));
    assert!(nicknames.contains("14"));

    let skus: std::collections::BTreeSet<&str> =
        rows.iter().map(|row| row.supplier_sku.as_str()).collect();
    assert_eq!(skus.len(), 35);
    assert!(skus.contains("107_san"));
    assert!(skus.contains("21061bhb"));
    assert!(skus.contains("115"));
}

#[test]
fn the_unlabelled_fifteenth_column_is_read() {
    let rows = sample_rows();
    assert!(rows.iter().all(|row| row.region == "Stavanger"));
}

#[test]
fn the_first_row_reads_exactly() {
    let rows = sample_rows();
    let first = &rows[0];

    assert_eq!(first.excel_row, 2);
    assert_eq!(first.order_id, 1_000_620_628);
    assert_eq!(first.quantity, 10);
    assert_eq!(first.product_id, 431);
    assert_eq!(first.product_name, "Barnehagebrødet - Oppskåret 750g");
    assert_eq!(first.supplier_sku, "10828");
    assert_eq!(first.position, "X-Bakehuset");
    assert_eq!(first.supplier, "bakehuset");
    assert_eq!(first.customer, "Customer 001");
    assert_eq!(first.department, None);
    assert_eq!(first.delivery_street, "Street 01");
    assert_eq!(first.comment, None);
    assert_eq!(first.route_nickname, "hau 2");
    assert_eq!(first.route_ordering, 0);
    assert!(first.accept_alternatives);
    assert_eq!(first.region, "Stavanger");
}
