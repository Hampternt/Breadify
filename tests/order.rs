//! Folding the real export's 352 lines into its 148 orders.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use breadify::order::{self, Order};
use support::sample_rows;

fn sample_orders() -> Vec<Order> {
    order::fold(&sample_rows())
}

#[test]
fn three_hundred_and_fifty_two_lines_are_one_hundred_and_forty_eight_orders() {
    let orders = sample_orders();

    assert_eq!(orders.len(), 148);
    assert_eq!(
        orders.iter().map(|order| order.lines.len()).sum::<usize>(),
        352
    );
}

#[test]
fn lines_per_order_match_the_export() {
    let orders = sample_orders();
    let mut histogram: BTreeMap<usize, usize> = BTreeMap::new();
    for order in &orders {
        *histogram.entry(order.lines.len()).or_default() += 1;
    }

    assert_eq!(
        histogram,
        BTreeMap::from([(1, 43), (2, 44), (3, 36), (4, 18), (5, 3), (6, 2), (7, 2)])
    );
}

#[test]
fn a_product_appears_at_most_once_per_order() {
    for order in sample_orders() {
        let distinct: BTreeSet<u32> = order.lines.iter().map(|line| line.product.id).collect();
        assert_eq!(
            distinct.len(),
            order.lines.len(),
            "order {} repeats a product",
            order.id
        );
    }
}

#[test]
fn orders_spanning_both_bakeries_stay_whole() {
    let mixed = sample_orders()
        .iter()
        .filter(|order| order.suppliers().len() > 1)
        .count();

    assert_eq!(mixed, 37);
}

#[test]
fn a_comment_is_carried_once_not_once_per_line() {
    let orders = sample_orders();
    let commented: Vec<&Order> = orders
        .iter()
        .filter(|order| order.comment.is_some())
        .collect();

    assert_eq!(commented.len(), 5);
    let acsenteret = commented
        .iter()
        .find(|order| order.id == 1_000_622_329)
        .expect("ACSENTERET's order carries a comment");
    assert_eq!(acsenteret.lines.len(), 4);
    // The comments are placeholders since the samples were anonymised, so
    // what is worth asserting is that one reached the order once — not what
    // it says.
    assert!(
        acsenteret
            .comment
            .as_deref()
            .is_some_and(|comment| comment.starts_with("Note "))
    );
}

#[test]
fn a_department_is_its_own_order() {
    let orders = sample_orders();
    let customer_012: Vec<&Order> = orders
        .iter()
        .filter(|order| order.customer == "Customer 012")
        .collect();

    assert_eq!(customer_012.len(), 9, "nine departments, nine orders");
    assert!(
        customer_012
            .iter()
            .all(|order| order.delivery_street == "Street 12")
    );
    assert!(customer_012.iter().all(|order| order.department.is_some()));

    let mut units: Vec<u32> = customer_012.iter().map(|order| order.units()).collect();
    units.sort_unstable();
    assert_eq!(units, [4, 4, 5, 6, 7, 7, 11, 12, 21]);
}

#[test]
fn unsequenced_orders_are_recognised() {
    let orders = sample_orders();
    let unsequenced: Vec<&Order> = orders
        .iter()
        .filter(|order| !order.is_sequenced())
        .collect();

    assert_eq!(unsequenced.len(), 20, "orders with no position assigned");
    assert_eq!(
        unsequenced
            .iter()
            .map(|order| order.lines.len())
            .sum::<usize>(),
        37,
        "lines on those orders"
    );
}
