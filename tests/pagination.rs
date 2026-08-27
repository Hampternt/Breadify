//! Paginating the sample day.

mod support;

use breadify::crates::CrateRules;
use breadify::geometry::FOOTER_CLEARANCE;
use breadify::layout::{self, Sheet};
use breadify::page::Primitive;
use breadify::route::{self, Route};
use breadify::{order, pdf};
use support::sample_rows;

fn routes() -> Vec<Route> {
    route::group(order::fold(&sample_rows()))
}

fn day() -> Vec<Sheet> {
    layout::day(
        &routes(),
        None,
        &CrateRules::default(),
        "PSR-BREAD-2026-03-04",
    )
}

#[test]
fn no_sheet_carries_two_routes() {
    let sheets = day();
    for sheet in &sheets {
        let mastheads = sheet
            .content
            .primitives
            .iter()
            .filter(
                |primitive| matches!(primitive, Primitive::Text { text, .. } if text == "ROUTE"),
            )
            .count();
        assert_eq!(mastheads, 1, "route {} sheet {}", sheet.route, sheet.number);
    }

    let order: Vec<&str> = sheets.iter().map(|sheet| sheet.route.as_str()).collect();
    let mut seen: Vec<&str> = Vec::new();
    for route in order {
        if seen.last() != Some(&route) {
            assert!(!seen.contains(&route), "route {route} is not contiguous");
            seen.push(route);
        }
    }
    assert_eq!(seen.len(), 16);
}

#[test]
fn every_route_starts_a_fresh_sheet_and_numbers_its_own_pages() {
    for sheet in day() {
        assert!(sheet.number >= 1 && sheet.number <= sheet.of);
    }

    let sheets = day();
    let first: Vec<usize> = sheets
        .iter()
        .filter(|sheet| sheet.number == 1)
        .map(|sheet| sheet.of)
        .collect();
    assert_eq!(first.len(), 16, "one first page per route");
}

#[test]
fn every_page_keeps_its_clearance_above_the_footer() {
    let limit = layout::footer_rule_y() - FOOTER_CLEARANCE;

    for sheet in day() {
        let lowest = sheet
            .content
            .primitives
            .iter()
            .filter(|primitive| !is_footer(primitive))
            .map(bottom_of)
            .fold(0.0_f64, f64::max);

        assert!(
            lowest <= limit,
            "route {} sheet {} of {} reaches {lowest:.1} mm, past the {limit:.1} mm limit",
            sheet.route,
            sheet.number,
            sheet.of
        );
    }
}

#[test]
fn a_stop_never_splits_across_a_break() {
    // Every block rule (1.25 pt, black) opens a stop; every sheet must carry
    // as many bread lines as its stops have, which the clearance test would
    // not catch on its own.
    let sheets = day();
    let stops: usize = sheets
        .iter()
        .map(|sheet| {
            sheet
                .content
                .primitives
                .iter()
                .filter(|primitive| {
                    matches!(primitive, Primitive::Rule { weight, .. } if (*weight - 1.25).abs() < 0.01)
                })
                .count()
        })
        .sum();

    // 148 stops, plus one 1.25 pt rule under each supplier column head of each
    // route's total.
    let column_heads: usize = routes()
        .iter()
        .map(|route| breadify::total::of(route).columns.len())
        .sum();
    assert_eq!(stops, 148 + column_heads);
}

#[test]
fn the_unsequenced_flag_is_never_the_last_thing_on_a_page() {
    for sheet in day() {
        let flag = sheet.content.primitives.iter().position(|primitive| {
            matches!(primitive, Primitive::Text { text, .. } if text.starts_with("NO POSITION"))
        });
        let Some(flag) = flag else {
            continue;
        };
        let below = sheet.content.primitives[flag..]
            .iter()
            .filter(|primitive| !is_footer(primitive))
            .count();
        assert!(
            below > 3,
            "route {} sheet {} ends on the flag",
            sheet.route,
            sheet.number
        );
    }
}

#[test]
fn the_whole_day_draws_to_one_pdf() {
    let sheets = day();
    let pages: Vec<_> = sheets.iter().map(|sheet| sheet.content.clone()).collect();
    let bytes = pdf::render(&pages, "Breadify pick lists").expect("the day renders");

    assert!(bytes.starts_with(b"%PDF"));
    assert!(
        sheets.len() >= 16,
        "at least one sheet per route, got {}",
        sheets.len()
    );
}

fn is_footer(primitive: &Primitive) -> bool {
    bottom_of(primitive) > layout::footer_rule_y() - 0.01
}

fn bottom_of(primitive: &Primitive) -> f64 {
    match primitive {
        Primitive::Text { baseline_start, .. } => baseline_start.y,
        Primitive::Rule { from, to, .. } => from.y.max(to.y),
        Primitive::Box { rect, .. } => rect.bottom(),
    }
}
