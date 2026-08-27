//! The total that closes each route's last page.
//!
//! How much of each bread the whole route needs — a cross-check for the
//! bakery rather than picking work (decision D15). See `docs/print-spec.md`
//! §7.

use std::collections::HashMap;

use crate::order::Product;
use crate::route::Route;

/// The order supplier columns print in. Anything not named here follows,
/// alphabetically.
pub const SUPPLIER_ORDER: [&str; 2] = ["sandnes bakeri", "bakehuset"];

/// How many of one bread the route needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotalLine {
    pub product: Product,
    pub units: u32,
    /// Full tens *inside a single order* — how many trays can be pulled whole.
    /// An order of 11 and an order of 9 make one, not two.
    pub full_tens: u32,
}

/// One bakery's column of the total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplierColumn {
    pub supplier: String,
    /// Most needed first, ties broken by name so two runs match.
    pub lines: Vec<TotalLine>,
}

impl SupplierColumn {
    /// Distinct breads in this column.
    pub fn types(&self) -> usize {
        self.lines.len()
    }

    /// Breads in this column, counted individually.
    pub fn units(&self) -> u32 {
        self.lines.iter().map(|line| line.units).sum()
    }
}

/// Everything the route needs, split by bakery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteTotal {
    pub columns: Vec<SupplierColumn>,
}

impl RouteTotal {
    /// Distinct breads across every column.
    pub fn types(&self) -> usize {
        self.columns.iter().map(SupplierColumn::types).sum()
    }

    /// Breads across every column, counted individually.
    pub fn units(&self) -> u32 {
        self.columns.iter().map(SupplierColumn::units).sum()
    }

    /// Full trays across the whole route.
    pub fn full_tens(&self) -> u32 {
        self.columns
            .iter()
            .flat_map(|column| column.lines.iter())
            .map(|line| line.full_tens)
            .sum()
    }
}

/// Totals one route.
pub fn of(route: &Route) -> RouteTotal {
    let mut by_product: HashMap<u32, TotalLine> = HashMap::new();

    for line in route.stops.iter().flat_map(|stop| stop.lines.iter()) {
        let entry = by_product
            .entry(line.product.id)
            .or_insert_with(|| TotalLine {
                product: line.product.clone(),
                units: 0,
                full_tens: 0,
            });
        entry.units += line.quantity;
        entry.full_tens += line.quantity / 10;
    }

    let mut by_supplier: HashMap<String, Vec<TotalLine>> = HashMap::new();
    for line in by_product.into_values() {
        by_supplier
            .entry(line.product.supplier.clone())
            .or_default()
            .push(line);
    }

    let mut columns: Vec<SupplierColumn> = by_supplier
        .into_iter()
        .map(|(supplier, mut lines)| {
            lines.sort_by(|left, right| {
                right
                    .units
                    .cmp(&left.units)
                    .then_with(|| left.product.name.cmp(&right.product.name))
            });
            SupplierColumn { supplier, lines }
        })
        .collect();

    columns.sort_by(|left, right| {
        supplier_position(&left.supplier).cmp(&supplier_position(&right.supplier))
    });

    RouteTotal { columns }
}

/// Where a supplier's column sits: the house order first, then anything new,
/// alphabetically.
fn supplier_position(supplier: &str) -> (usize, &str) {
    let known = SUPPLIER_ORDER
        .iter()
        .position(|name| name.eq_ignore_ascii_case(supplier));
    (known.unwrap_or(SUPPLIER_ORDER.len()), supplier)
}

/// `6 types · 33 units`, pluralised — a route with one bread reads
/// `1 type · 10 units`.
pub fn summary(types: usize, units: u32) -> String {
    format!(
        "{types} {} · {units} {}",
        plural(types as u32, "type"),
        plural(units, "unit")
    )
}

fn plural(count: u32, word: &str) -> String {
    if count == 1 {
        return word.to_owned();
    }
    format!("{word}s")
}
