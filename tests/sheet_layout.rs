//! Laying route 8 out on a sheet, and drawing it to PDF.

mod support;

use breadify::geometry::{CONTENT_WIDTH, MARGIN_SIDE, PAGE_HEIGHT, PAGE_WIDTH, Rect};
use breadify::layout::metrics::{BADGE_PADDING, CRATE_GLYPH, RULE_DEPARTMENT_BOX};
use breadify::layout::{self, Cursor, MarkerTreatment, Settings, SheetContext, stop};
use breadify::order::{Line, Order, Product};
use breadify::page::{BRAND_RED, Page, Primitive};
use breadify::route::{self, Route};
use breadify::text;
use breadify::{order, pdf};
use support::{freezer_rows, sample_rows};

fn routes() -> Vec<Route> {
    route::group(order::fold(&sample_rows()))
}

fn freezer_routes() -> Vec<Route> {
    route::group(order::fold(&freezer_rows()))
}

fn named(routes: &[Route], nickname: &str) -> Route {
    routes
        .iter()
        .find(|route| route.nickname == nickname)
        .expect("route is in the sample")
        .clone()
}

fn sheet_of(route: &Route) -> Page {
    let context = SheetContext::single(route, None, "PSR-BREAD-2026-03-04");
    layout::sheet(route, &context, &Settings::default())
}

/// Every sheet a route needs — most fit on one, some do not.
fn sheets_of(route: &Route) -> Vec<Page> {
    layout::paginate(route, None, &Settings::default(), "PSR-BREAD-2026-03-04")
        .into_iter()
        .map(|sheet| sheet.content)
        .collect()
}

fn runs(page: &Page) -> Vec<&str> {
    page.primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn every_stop_and_every_bread_reaches_the_sheet() {
    let routes = routes();
    let route = named(&routes, "8");
    let page = sheet_of(&route);
    let set = runs(&page);

    for stop in &route.stops {
        assert!(
            set.contains(&stop.customer.as_str()),
            "{} is missing",
            stop.customer
        );
        for line in &stop.lines {
            assert!(
                set.contains(&line.product.name.as_str()),
                "{} is missing",
                line.product.name
            );
        }
    }
}

#[test]
fn product_names_are_set_verbatim() {
    let routes = routes();
    let page = sheet_of(&named(&routes, "8"));
    assert!(runs(&page).contains(&"Grovbrød M/sirup Oppdelt Sandnes Bakeri"));
}

#[test]
fn nothing_is_drawn_outside_the_sheet() {
    for route in routes() {
        for page in sheets_of(&route) {
            for primitive in &page.primitives {
                let (x, y) = match primitive {
                    Primitive::Text { baseline_start, .. } => (baseline_start.x, baseline_start.y),
                    Primitive::Rule { from, to, .. } => (from.x.min(to.x), from.y.min(to.y)),
                    Primitive::Artwork { rect, .. } | Primitive::Box { rect, .. } => {
                        (rect.x, rect.y)
                    }
                };
                assert!(
                    (MARGIN_SIDE - 0.001..=PAGE_WIDTH - MARGIN_SIDE).contains(&x),
                    "route {} draws at x={x:.2}",
                    route.nickname
                );
                assert!(
                    (0.0..=PAGE_HEIGHT).contains(&y),
                    "route {} draws at y={y:.2}",
                    route.nickname
                );
            }
        }
    }
}

#[test]
fn the_masthead_carries_one_brand_rule() {
    let routes = routes();
    let page = sheet_of(&named(&routes, "8"));
    let brand = page
        .primitives
        .iter()
        .filter(
            |primitive| matches!(primitive, Primitive::Rule { colour, .. } if *colour == BRAND_RED),
        )
        .count();

    assert_eq!(brand, 1);
}

#[test]
fn the_unsequenced_flag_appears_once_and_only_where_it_is_needed() {
    for route in routes() {
        let flags: usize = sheets_of(&route)
            .iter()
            .map(|page| {
                runs(page)
                    .iter()
                    .filter(|run| run.starts_with("NO POSITION ASSIGNED"))
                    .count()
            })
            .sum();
        let expected = usize::from(route.unsequenced().count() > 0);

        assert_eq!(flags, expected, "route {}", route.nickname);
    }
}

/// The words print whatever the treatment; the bar is what the badge adds,
/// and it is not what a page prints unless the user asks for it.
#[test]
fn refusing_substitutes_says_so_and_only_the_badge_adds_a_bar() {
    let route = named(&routes(), "8");
    let heavy = |settings: &Settings| {
        let context = SheetContext::single(&route, None, "PSR-BREAD-2026-03-04");
        let page = layout::sheet(&route, &context, settings);
        assert!(runs(&page).contains(&"WANT SUBSTITUTE: FALSE"));
        page.primitives
            .iter()
            .filter(
                |primitive| matches!(primitive, Primitive::Rule { weight, .. } if *weight >= 5.0),
            )
            .count()
    };

    assert_eq!(heavy(&Settings::default()), 0, "the default is words alone");
    assert_eq!(
        heavy(&Settings {
            marker: MarkerTreatment::InvertedBadge,
            ..Settings::default()
        }),
        1,
        "one block on route 8 refuses substitutes"
    );
}

#[test]
fn a_short_route_leaves_the_footer_alone() {
    let routes = routes();
    let page = sheet_of(&named(&routes, "8"));
    assert!(
        page.lowest_point() < PAGE_HEIGHT,
        "route 8 should fit on one sheet"
    );
}

#[test]
fn every_route_draws_to_an_a4_pdf() {
    for route in routes() {
        let bytes = pdf::render(&sheets_of(&route), &format!("Route {}", route.nickname))
            .expect("the sheets should render");

        assert!(bytes.starts_with(b"%PDF"), "route {}", route.nickname);
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("MediaBox[0 0 595.27563 841.88983]"),
            "route {} should have an A4 media box",
            route.nickname
        );
    }
}

#[test]
fn norwegian_letters_survive_the_pdf() {
    let routes = routes();
    let page = sheet_of(&named(&routes, "8"));
    let bytes = pdf::render(&[page], "Route 8").expect("renders");

    let directory = std::env::temp_dir().join("breadify-round-trip");
    std::fs::create_dir_all(&directory).expect("a temporary directory");
    let file = directory.join("route-8.pdf");
    std::fs::write(&file, &bytes).expect("writing the pdf");

    let Ok(output) = std::process::Command::new("pdftotext")
        .arg(&file)
        .arg("-")
        .output()
    else {
        println!("pdftotext is not installed — skipping the round trip");
        return;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for word in ["KJØKKEN", "Rugbrød", "Sekskornsbrød", "RAVNÅS"] {
        assert!(text.contains(word), "{word} did not survive the round trip");
    }
}

/// Everything a heading draws, as labelled rectangles.
///
/// Checked pairwise: no two may overlap, and none may leave the column. That
/// is the whole invariant — it does not care which line anything landed on,
/// which is what makes it hold for arrangements nobody thought of.
///
/// Returns true when the crates did not fit beside the marker and went below.
fn heading_holds_together(order: &Order, settings: &Settings) -> bool {
    let (page, _) = stop::block(order, settings, &Cursor::new(0.0));
    let where_ = format!("{} under {:?}", order.customer, settings.marker);
    let mut boxes: Vec<(&str, Rect)> = Vec::new();

    for primitive in &page.primitives {
        match primitive {
            Primitive::Text {
                baseline_start,
                text,
                style,
                ..
            } => {
                let label = if *text == order.customer {
                    "the customer name"
                } else if text.starts_with("want substitute") || text.starts_with("WANT SUBSTITUTE")
                {
                    "the substitute marker"
                } else if *text == order.id.to_string() {
                    "the order id"
                } else {
                    continue;
                };

                let pad = if label == "the substitute marker" && settings.marker.has_badge() {
                    BADGE_PADDING.1
                } else {
                    0.0
                };
                boxes.push((
                    label,
                    Rect::new(
                        baseline_start.x - pad,
                        baseline_start.y - text::ascent(*style),
                        text::width(text, *style) + 2.0 * pad,
                        text::line_height(*style),
                    ),
                ));
            }
            Primitive::Box { rect, stroke, .. } => {
                // A half crate is drawn twice — the outline, and the fill in
                // its lower half. Only the outline is the glyph.
                if (rect.width - CRATE_GLYPH.0).abs() < 0.01
                    && (rect.height - CRATE_GLYPH.1).abs() < 0.01
                {
                    boxes.push(("a crate", *rect));
                } else if stroke
                    .is_some_and(|stroke| (stroke.weight - RULE_DEPARTMENT_BOX).abs() < 0.01)
                {
                    boxes.push(("the DPT box", *rect));
                }
            }
            _ => {}
        }
    }

    assert!(
        boxes.iter().any(|(label, _)| *label == "the customer name"),
        "{where_}: the customer name is on the block"
    );
    assert!(
        boxes
            .iter()
            .any(|(label, _)| *label == "the substitute marker"),
        "{where_}: the marker is on the block"
    );
    assert_eq!(
        boxes.iter().any(|(label, _)| *label == "the DPT box"),
        order.department.is_some(),
        "{where_}: the DPT box appears exactly when there is a department"
    );

    // The DPT box is drawn hard against the block's left edge, so its rule sits
    // a hair outside the column; everything else keeps within it.
    for (label, rect) in &boxes {
        assert!(
            rect.x >= MARGIN_SIDE - 0.01 && rect.right() <= MARGIN_SIDE + CONTENT_WIDTH + 0.01,
            "{where_}: {label} runs {:.2}..{:.2}, outside the column",
            rect.x,
            rect.right()
        );
    }

    for (index, (label, one)) in boxes.iter().enumerate() {
        for (other_label, two) in &boxes[index + 1..] {
            let across = one.x < two.right() - 0.01 && two.x < one.right() - 0.01;
            let down = one.y < two.bottom() - 0.01 && two.y < one.bottom() - 0.01;
            assert!(
                !(across && down),
                "{where_}: {label} ({:.2}..{:.2} x {:.2}..{:.2}) overlaps {other_label} \
                 ({:.2}..{:.2} x {:.2}..{:.2})",
                one.x,
                one.right(),
                one.y,
                one.bottom(),
                two.x,
                two.right(),
                two.y,
                two.bottom()
            );
        }
    }

    let name_bottom = boxes
        .iter()
        .find(|(label, _)| *label == "the customer name")
        .map_or(0.0, |(_, rect)| rect.bottom());
    boxes
        .iter()
        .any(|(label, rect)| *label == "a crate" && rect.y >= name_bottom - 0.01)
}

/// The heading places the crate glyphs by measurement, against the marker on
/// its right and the customer name on its left. Five of the sample's 148 stops
/// have a name long enough to reach them; those put their crates below.
#[test]
fn no_heading_runs_into_its_own_right_hand_group() {
    let orders = order::fold(&sample_rows());
    let mut dropped = 0;

    for treatment in MarkerTreatment::ALL {
        let settings = Settings {
            marker: treatment,
            ..Settings::default()
        };
        for order in &orders {
            if heading_holds_together(order, &settings) && treatment == MarkerTreatment::default() {
                dropped += 1;
            }
        }
    }

    assert_eq!(
        dropped, 5,
        "five sample names are long enough to push their crates down a line"
    );
}

/// The export cannot reach the hard cases on its own: nothing in it puts a
/// long name, a long department and a big crate count on one stop. The
/// bakery's own size modifiers can, and did — before this was fixed, a 14-crate
/// stop with a 38-character department drew 27 mm of crates through its own
/// DPT box.
#[test]
fn a_heading_with_nowhere_left_to_put_the_crates_still_does_not_collide() {
    let longest_name = "Customer 001";
    let longest_department = "Department 10";

    let cases = [
        (
            "long name, long department, 14 crates",
            longest_name,
            Some(longest_department),
            140,
        ),
        (
            "long department, 40 crates — more than a row",
            "Customer 012",
            Some(longest_department),
            400,
        ),
        (
            "no department, more crates than the column holds",
            longest_name,
            None,
            300,
        ),
        ("nothing unusual at all", "Customer 024", None, 3),
        (
            "one bread, one crate, a department",
            "Customer 037",
            Some("Department 31"),
            1,
        ),
    ];

    for (what, customer, department, quantity) in cases {
        for treatment in MarkerTreatment::ALL {
            for show_order_id in [true, false] {
                let settings = Settings {
                    marker: treatment,
                    show_order_id,
                    ..Settings::default()
                };
                let order = synthetic(customer, department, quantity);
                println!("{what} · {treatment:?} · order id {show_order_id}");
                heading_holds_together(&order, &settings);
            }
        }
    }
}

/// A stop built by hand, for the shapes the export does not happen to contain.
fn synthetic(customer: &str, department: Option<&str>, quantity: u32) -> Order {
    Order {
        id: 1_000_622_147,
        customer: customer.to_owned(),
        department: department.map(str::to_owned),
        delivery_street: "Lassaveien 10".to_owned(),
        route: "13".to_owned(),
        sequence: 600,
        accept_alternatives: false,
        comment: None,
        lines: vec![Line {
            product: Product {
                id: 1,
                name: "Havrebrød Oppdelt Sandnes Bakeri".to_owned(),
                sku: "12345".to_owned(),
                supplier: "Sandnes Bakeri".to_owned(),
            },
            quantity,
        }],
    }
}

/// Everything the route total writes, as labelled rectangles, checked the same
/// way a heading is: none may leave the column, and no two may overlap.
///
/// The total lays one column per supplier side by side. Bread has two of them
/// and the arithmetic was written for two; the freezer list has seven on route
/// 8, which divided the page into strips too narrow for a product name.
fn total_holds_together(route: &Route) {
    let (page, _) = layout::total::block(route, &Cursor::new(0.0));
    let where_ = format!("route {} total", route.nickname);
    let mut boxes: Vec<(&str, Rect)> = Vec::new();

    for primitive in &page.primitives {
        let Primitive::Text {
            baseline_start,
            text: run,
            style,
            ..
        } = primitive
        else {
            continue;
        };
        // The title and the two meta lines each own a whole line of the block;
        // it is the columns underneath that can collide.
        if run.ends_with("most to least")
            || run.starts_with("Route ")
            || run.starts_with("one full ten")
        {
            continue;
        }
        boxes.push((
            run.as_str(),
            Rect::new(
                baseline_start.x,
                baseline_start.y - text::ascent(*style),
                text::width(run, *style),
                text::line_height(*style),
            ),
        ));
    }

    assert!(!boxes.is_empty(), "{where_}: the total wrote nothing");

    for (label, rect) in &boxes {
        assert!(
            rect.x >= MARGIN_SIDE - 0.01 && rect.right() <= MARGIN_SIDE + CONTENT_WIDTH + 0.01,
            "{where_}: {label:?} runs {:.2}..{:.2}, outside the {CONTENT_WIDTH} mm column",
            rect.x,
            rect.right()
        );
    }

    for (index, (label, one)) in boxes.iter().enumerate() {
        for (other, two) in &boxes[index + 1..] {
            let across = one.x < two.right() - 0.01 && two.x < one.right() - 0.01;
            let down = one.y < two.bottom() - 0.01 && two.y < one.bottom() - 0.01;
            assert!(
                !(across && down),
                "{where_}: {label:?} ({:.2}..{:.2}) overlaps {other:?} ({:.2}..{:.2})",
                one.x,
                one.right(),
                two.x,
                two.right()
            );
        }
    }
}

#[test]
fn every_bread_route_total_holds_together() {
    for route in routes() {
        total_holds_together(&route);
    }
}

#[test]
fn every_freezer_route_total_holds_together() {
    for route in freezer_routes() {
        total_holds_together(&route);
    }
}
