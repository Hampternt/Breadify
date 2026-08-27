//! Laying route 8 out on a sheet, and drawing it to PDF.

mod support;

use breadify::crates::CrateRules;
use breadify::geometry::{MARGIN_SIDE, PAGE_HEIGHT, PAGE_WIDTH};
use breadify::layout::{self, SheetContext};
use breadify::page::{BRAND_RED, Page, Primitive};
use breadify::route::{self, Route};
use breadify::{order, pdf};
use support::sample_rows;

fn routes() -> Vec<Route> {
    route::group(order::fold(&sample_rows()))
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
    layout::sheet(route, &context, &CrateRules::default())
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
        let page = sheet_of(&route);
        for primitive in &page.primitives {
            let (x, y) = match primitive {
                Primitive::Text { baseline_start, .. } => (baseline_start.x, baseline_start.y),
                Primitive::Rule { from, to, .. } => (from.x.min(to.x), from.y.min(to.y)),
                Primitive::Box { rect, .. } => (rect.x, rect.y),
            };
            assert!(
                (MARGIN_SIDE - 0.001..=PAGE_WIDTH - MARGIN_SIDE).contains(&x),
                "route {} draws at x={x:.2}",
                route.nickname
            );
            assert!(y >= 0.0, "route {} draws above the sheet", route.nickname);
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
        let page = sheet_of(&route);
        let flags = runs(&page)
            .iter()
            .filter(|run| run.starts_with("NO POSITION ASSIGNED"))
            .count();
        let expected = usize::from(route.unsequenced().count() > 0);

        assert_eq!(flags, expected, "route {}", route.nickname);
    }
}

#[test]
fn refusing_substitutes_shows_the_badge_and_the_bar() {
    let routes = routes();
    let page = sheet_of(&named(&routes, "8"));

    assert!(runs(&page).contains(&"WANT SUBSTITUTE: FALSE"));
    let heavy = page
        .primitives
        .iter()
        .filter(|primitive| matches!(primitive, Primitive::Rule { weight, .. } if *weight >= 5.0))
        .count();
    assert_eq!(heavy, 1, "one block on route 8 refuses substitutes");
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
        let page = sheet_of(&route);
        let bytes = pdf::render(&[page], &format!("Route {}", route.nickname))
            .expect("the sheet should render");

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
