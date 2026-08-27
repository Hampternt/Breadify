//! Folding order lines into orders.
//!
//! A row of the export is one product on one order; everything else on the row
//! belongs to the order and is repeated onto each of its lines. An order is
//! also a stop and a crate label: one customer, or one department of a
//! customer (decision D16).

use std::collections::HashMap;

use crate::sheet::SheetRow;

/// A bread, as the export identifies it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    pub id: u32,
    /// Printed exactly as the file has it, trailing bakery name and all
    /// (decision D14).
    pub name: String,
    pub sku: String,
    pub supplier: String,
}

/// One bread on one order, and how many of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub product: Product,
    pub quantity: u32,
}

/// One order: one stop, one crate label, one block on the printed page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: i64,
    pub customer: String,
    /// The sub-location inside a customer. Absent for most orders; where it is
    /// present it is what gets written on the crate.
    pub department: Option<String>,
    pub delivery_street: String,
    /// The route nickname exactly as the file spells it — `7`, `hau 2`. Never
    /// the integer parsed out of it (decision D12).
    pub route: String,
    /// Where the stop falls in the route's delivery order, higher being later.
    /// `0` means no position was assigned; see [`is_sequenced`](Self::is_sequenced).
    pub sequence: u32,
    /// Whether a replacement product is acceptable when a bread is sold out.
    pub accept_alternatives: bool,
    /// The customer's note, carried once rather than once per line.
    pub comment: Option<String>,
    /// In the order the export lists them.
    pub lines: Vec<Line>,
}

impl Order {
    /// Total breads on the order, across every line. This is what the crate
    /// arithmetic counts.
    pub fn units(&self) -> u32 {
        self.lines.iter().map(|line| line.quantity).sum()
    }

    /// Whether the export gave this stop a position in its route.
    ///
    /// Unsequenced stops print after the sequenced ones, under a flag, because
    /// nobody chose to put them last — see `docs/print-spec.md` §6.
    pub fn is_sequenced(&self) -> bool {
        self.sequence != 0
    }

    /// The bakeries this order draws from. Usually one; 37 of the sample's 148
    /// orders draw from both, and stay one stop on one list regardless.
    pub fn suppliers(&self) -> Vec<&str> {
        let mut suppliers: Vec<&str> = self
            .lines
            .iter()
            .map(|line| line.product.supplier.as_str())
            .collect();
        suppliers.sort_unstable();
        suppliers.dedup();
        suppliers
    }
}

/// Folds rows into orders, keeping both the orders and their lines in the
/// order the file lists them.
///
/// Order-level values are taken from the order's first line. Lines that
/// disagree are a defect in the export rather than something to resolve here;
/// [`crate::validate::run`] reports them, and it should run first.
pub fn fold(rows: &[SheetRow]) -> Vec<Order> {
    let mut orders: Vec<Order> = Vec::new();
    let mut position_of: HashMap<i64, usize> = HashMap::new();

    for row in rows {
        let line = Line {
            product: Product {
                id: row.product_id,
                name: row.product_name.clone(),
                sku: row.supplier_sku.clone(),
                supplier: row.supplier.clone(),
            },
            quantity: row.quantity,
        };

        if let Some(&position) = position_of.get(&row.order_id) {
            let order: &mut Order = &mut orders[position];
            order.lines.push(line);
            if order.comment.is_none() {
                order.comment = row.comment.clone();
            }
            continue;
        }

        position_of.insert(row.order_id, orders.len());
        orders.push(Order {
            id: row.order_id,
            customer: row.customer.clone(),
            department: row.department.clone(),
            delivery_street: row.delivery_street.clone(),
            route: row.route_nickname.clone(),
            sequence: row.route_ordering,
            accept_alternatives: row.accept_alternatives,
            comment: row.comment.clone(),
            lines: vec![line],
        });
    }

    orders
}
