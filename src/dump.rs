//! Printing a route to the terminal in the shape of the worked examples.
//!
//! Not the printed page — that is pack 2 — but the same content, so the model
//! can be read and checked against `docs/print-spec.md` §10 before any of it
//! is drawn.

use std::fmt::Write as _;

use crate::crates::{self, CrateRules};
use crate::date::DeliveryDates;
use crate::order::Order;
use crate::route::Route;
use crate::supplier;
use crate::total;

/// Renders one route: its stops in delivery order, the unsequenced ones under
/// a flag, then the route total.
pub fn route(route: &Route, dates: Option<DeliveryDates>, rules: &CrateRules) -> String {
    let mut out = String::new();
    let date = dates.map_or_else(|| "date unknown".to_owned(), |dates| dates.to_string());

    let _ = writeln!(
        out,
        "ROUTE {} — {date} — {} stops, {} lines",
        route.nickname,
        route.stops.len(),
        route.line_count()
    );

    let mut flagged = false;
    for stop in &route.stops {
        if !stop.is_sequenced() && !flagged {
            flagged = true;
            let _ = writeln!(
                out,
                "\n─── no position assigned — driver decides the order ───"
            );
        }
        let _ = write!(out, "\n{}", stop_block(stop, rules));
    }

    let _ = write!(out, "\n{}", route_total(route));
    out
}

/// One stop: the heading a crate label is copied from, then its bread.
fn stop_block(stop: &Order, rules: &CrateRules) -> String {
    let mut out = String::new();
    let heading = match &stop.department {
        Some(department) => format!("{} — {department}", stop.customer),
        None => stop.customer.clone(),
    };

    let _ = writeln!(
        out,
        "{heading}  {}  {}  {}",
        crate_glyphs(stop, rules),
        substitute_marker(stop),
        stop.id
    );

    for line in &stop.lines {
        let _ = writeln!(
            out,
            "  {:>3}  {}  {}",
            line.quantity,
            supplier::code(&line.product.supplier),
            line.product.name
        );
    }

    if let Some(comment) = &stop.comment {
        let _ = writeln!(out, "       note: {comment}");
    }

    out
}

/// The route's closing cross-check, one column per bakery.
fn route_total(route: &Route) -> String {
    let total = total::of(route);
    let mut out = String::new();

    let _ = writeln!(
        out,
        "Route {} total — {}",
        route.nickname,
        total::summary(total.types(), total.units())
    );

    for column in &total.columns {
        let _ = writeln!(
            out,
            "  {} {} — {}",
            supplier::code(&column.supplier),
            supplier::display_name(&column.supplier),
            total::summary(column.types(), column.units())
        );
        for line in &column.lines {
            let _ = writeln!(
                out,
                "    {:>3}  {}{}",
                line.units,
                line.product.name,
                "  ●".repeat(line.full_tens as usize)
            );
        }
    }

    out
}

/// `■` for a crate of ten, `◪` for a crate of five.
fn crate_glyphs(stop: &Order, rules: &CrateRules) -> String {
    let count = crates::count(stop, rules);
    let glyphs = "■ ".repeat(count.large as usize) + &"◪ ".repeat(count.small as usize);
    glyphs.trim_end().to_owned()
}

/// Quiet when substitutes are fine, loud when they are not — the state that
/// changes what a driver does.
fn substitute_marker(stop: &Order) -> &'static str {
    if stop.accept_alternatives {
        return "want substitute: true";
    }
    "WANT SUBSTITUTE: FALSE"
}
