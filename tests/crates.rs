//! Crate arithmetic against the real export.
//!
//! The figures here were recomputed from the file, not taken from the design
//! handoff — whose "one stop · nine crates" for Customer 012 counts
//! departments, not crates.

mod support;

use breadify::crates::{self, CrateCount, CrateRules};
use breadify::order::{self, Order};
use breadify::route;
use support::sample_rows;

fn sample_orders() -> Vec<Order> {
    order::fold(&sample_rows())
}

#[test]
fn route_eight_gets_the_crates_the_worked_example_shows() {
    let rules = CrateRules::default();
    let routes = route::group(sample_orders());
    let route_eight = routes.iter().find(|r| r.nickname == "8").unwrap();

    let counts: Vec<(u32, CrateCount)> = route_eight
        .stops
        .iter()
        .map(|stop| (stop.units(), crates::count(stop, &rules)))
        .collect();

    assert_eq!(
        counts,
        [
            (14, CrateCount { large: 1, small: 1 }), // Customer 024
            (9, CrateCount { large: 1, small: 0 }),  // TEQVA GRUPPEN
            (10, CrateCount { large: 1, small: 0 }), // TROLLHAUGEN
            (8, CrateCount { large: 1, small: 0 }),  // A3 TRANSPORT
            (2, CrateCount { large: 0, small: 1 }),  // IDSØ & RAVNÅS
        ]
    );
}

#[test]
fn customer_012_is_thirteen_crates_not_nine() {
    let rules = CrateRules::default();
    let orders = sample_orders();
    let customer_012: Vec<&Order> = orders
        .iter()
        .filter(|order| order.customer == "Customer 012")
        .collect();

    let total: u32 = customer_012
        .iter()
        .map(|order| crates::count(order, &rules).total())
        .sum();

    assert_eq!(customer_012.len(), 9, "nine departments");
    assert_eq!(total, 13, "thirteen crates");
}

#[test]
fn the_remainder_rule_prefers_the_fewest_containers() {
    let rules = CrateRules::default();
    let cases = [
        (1, CrateCount { large: 0, small: 1 }),
        (5, CrateCount { large: 0, small: 1 }),
        (6, CrateCount { large: 1, small: 0 }),
        (10, CrateCount { large: 1, small: 0 }),
        (11, CrateCount { large: 1, small: 1 }),
        (16, CrateCount { large: 2, small: 0 }),
        (20, CrateCount { large: 2, small: 0 }),
        (48, CrateCount { large: 5, small: 0 }),
    ];

    for (units, expected) in cases {
        let order = order_of(&[units]);
        assert_eq!(crates::count(&order, &rules), expected, "{units} units");
    }
}

#[test]
fn half_size_breads_take_half_a_slot() {
    let mut rules = CrateRules::default();
    // Product 1 is a roll; product 2 stays a standard bread.
    rules.set_size(1, 50);

    let rolls_only = order_of_products(&[(1, 12)]);
    assert_eq!(crates::slots(&rolls_only, &rules), 6);
    assert_eq!(
        crates::count(&rolls_only, &rules),
        CrateCount { large: 1, small: 0 }
    );

    let mixed = order_of_products(&[(1, 3), (2, 4)]);
    // 3 rolls fill one and a half slots, plus 4 standard = 5.5, rounded up.
    assert_eq!(crates::slots(&mixed, &rules), 6);
}

#[test]
fn a_bulky_bread_can_take_more_than_one_slot() {
    let mut rules = CrateRules::default();
    rules.set_size(1, 200);

    let order = order_of_products(&[(1, 6)]);
    assert_eq!(crates::slots(&order, &rules), 12);
    assert_eq!(
        crates::count(&order, &rules),
        CrateCount { large: 1, small: 1 }
    );
}

#[test]
fn standard_sizes_leave_the_verified_totals_untouched() {
    let mut rules = CrateRules::default();
    for product_id in 1..=9999 {
        rules.set_size(product_id, 100);
    }

    let orders = sample_orders();
    let with_explicit: u32 = orders
        .iter()
        .map(|order| crates::count(order, &rules).total())
        .sum();
    let with_defaults: u32 = orders
        .iter()
        .map(|order| crates::count(order, &CrateRules::default()).total())
        .sum();

    assert_eq!(with_explicit, with_defaults);
}

/// An order of one product per quantity given, for exercising the arithmetic.
fn order_of(quantities: &[u32]) -> Order {
    order_of_products(
        &quantities
            .iter()
            .enumerate()
            .map(|(index, quantity)| (index as u32 + 1, *quantity))
            .collect::<Vec<_>>(),
    )
}

fn order_of_products(lines: &[(u32, u32)]) -> Order {
    use breadify::order::{Line, Product};

    Order {
        id: 1,
        customer: "Test".to_owned(),
        department: None,
        delivery_street: "Testveien 1".to_owned(),
        route: "1".to_owned(),
        sequence: 100,
        accept_alternatives: true,
        comment: None,
        lines: lines
            .iter()
            .map(|(product_id, quantity)| Line {
                product: Product {
                    id: *product_id,
                    name: format!("Product {product_id}"),
                    sku: product_id.to_string(),
                    supplier: "sandnes bakeri".to_owned(),
                },
                quantity: *quantity,
            })
            .collect(),
    }
}

/// The size buttons are the language the Configure step speaks; the
/// arithmetic still runs on hundredths.
#[test]
fn every_preset_fraction_comes_to_the_slots_it_names() {
    use breadify::crates::{SIZE_PRESETS, STANDARD_SIZE, spoken};

    for (label, percent) in SIZE_PRESETS {
        assert_eq!(spoken(percent), label, "{label} reads back as itself");
    }
    assert_eq!(spoken(175), "175 %", "anything typed reads as a percentage");

    // Thirds round down, so three of them still fit one slot.
    let third = SIZE_PRESETS
        .iter()
        .find(|(label, _)| *label == "1/3")
        .expect("a third is one of the buttons")
        .1;
    assert_eq!(3 * third / STANDARD_SIZE, 0, "three thirds do not spill");
    assert!(3 * third < STANDARD_SIZE);
    assert!(4 * third > STANDARD_SIZE);
}

/// Setting a bread back to a whole slot forgets it rather than recording the
/// default, so "which breads has someone said something about" stays
/// answerable and a no-op change does not repaginate the day.
#[test]
fn saying_a_bread_is_standard_forgets_it() {
    use breadify::crates::{CrateRules, STANDARD_SIZE};

    let mut rules = CrateRules::default();
    assert!(!rules.is_custom(7));

    rules.set_size(7, 50);
    assert!(rules.is_custom(7));
    assert_eq!(rules.size_of(7), 50);

    rules.set_size(7, STANDARD_SIZE);
    assert!(!rules.is_custom(7));
    assert_eq!(rules.size_of(7), STANDARD_SIZE);
    assert_eq!(rules, CrateRules::default(), "and nothing is left behind");
}

/// One stop per quantity, each a single line of a standard bread, so a
/// route's crate total is easy to state in the test.
fn synthetic_route(nickname: &str, quantities: &[u32]) -> breadify::route::Route {
    use breadify::order::{Line, Product};

    breadify::route::Route {
        nickname: nickname.to_owned(),
        stops: quantities
            .iter()
            .enumerate()
            .map(|(index, &quantity)| Order {
                id: 1_000_700_000 + index as i64,
                customer: format!("Customer {index:03}"),
                department: None,
                delivery_street: format!("Gate {index}"),
                route: nickname.to_owned(),
                sequence: 100 * (index as u32 + 1),
                accept_alternatives: true,
                comment: None,
                lines: vec![Line {
                    product: Product {
                        id: 1,
                        name: "Havrebrød Oppdelt Sandnes Bakeri".to_owned(),
                        sku: "101".to_owned(),
                        supplier: "sandnes bakeri".to_owned(),
                    },
                    quantity,
                }],
            })
            .collect(),
    }
}

/// Everything every sheet of a route writes, one string per text run.
fn sheet_text(
    route: &breadify::route::Route,
    settings: &breadify::layout::Settings,
) -> Vec<String> {
    breadify::layout::day(std::slice::from_ref(route), None, settings, "PSR-BREAD")
        .iter()
        .flat_map(|sheet| &sheet.content.primitives)
        .filter_map(|primitive| match primitive {
            breadify::page::Primitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// A route's crates are summed across its stops, each stop's count first —
/// two stops of 14 units are four crates (1 large + 1 small each), not the
/// three that 28 pooled units would make.
#[test]
fn a_route_total_sums_stops_not_units() {
    let rules = CrateRules::default();
    assert_eq!(
        crates::route_total(&synthetic_route("6", &[14, 14]), &rules),
        4
    );
    assert_eq!(
        crates::route_total(&synthetic_route("6", &[90, 90]), &rules),
        18
    );
}

/// More than sixteen crates on one route and every sheet of it says to take
/// a pallet (decision D25); exactly sixteen does not.
#[test]
fn a_heavy_route_recommends_a_pallet_on_every_sheet() {
    let settings = breadify::layout::Settings::default();

    let heavy = synthetic_route("6", &[90, 90]);
    let sheets = breadify::layout::day(std::slice::from_ref(&heavy), None, &settings, "PSR-BREAD");
    for sheet in &sheets {
        let texts: Vec<String> = sheet
            .content
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                breadify::page::Primitive::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert!(
            texts
                .iter()
                .any(|text| text.contains("18 crates — take a pallet.")),
            "sheet {} of route 6 carries the pallet call",
            sheet.number
        );
    }

    let at_threshold = synthetic_route("6", &[80, 80]);
    assert_eq!(
        crates::route_total(&at_threshold, &CrateRules::default()),
        crates::PALLET_THRESHOLD
    );
    assert!(
        !sheet_text(&at_threshold, &settings)
            .iter()
            .any(|text| text.contains("pallet")),
        "sixteen crates is not more than sixteen"
    );
}

/// The freezer list has no crate arithmetic (decision F4), so it never asks
/// for a pallet, however big the route.
#[test]
fn the_freezer_list_never_recommends_a_pallet() {
    let settings = breadify::layout::Settings {
        kind: breadify::date::ExportKind::Freezer,
        ..breadify::layout::Settings::default()
    };
    assert!(
        !sheet_text(&synthetic_route("6", &[90, 90]), &settings)
            .iter()
            .any(|text| text.contains("pallet"))
    );
}
