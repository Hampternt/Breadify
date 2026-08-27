//! Laying route 8 out on a sheet, and drawing it to PDF.

mod support;

use breadify::geometry::{MARGIN_SIDE, PAGE_HEIGHT, PAGE_WIDTH};
use breadify::layout::Settings;
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

/// The heading places the crate glyphs by measurement, against the marker on
/// its right and the customer name on its left. Five of the sample's 148 stops
/// have a name long enough to reach them; those drop their crates to the
/// second line. Nothing may overlap either way.
#[test]
fn no_heading_runs_into_its_own_right_hand_group() {
    use breadify::font::Face;
    use breadify::layout::metrics::{
        BADGE_PADDING, CRATE_GAP, CRATE_GLYPH, MARKER_GAP, RULE_DEPARTMENT_BOX, SIZE_CUSTOMER,
        TRACK_CUSTOMER,
    };
    use breadify::layout::{Cursor, MarkerTreatment, stop};
    use breadify::page::Stroke;
    use breadify::text::{self, Style};

    let orders = order::fold(&sample_rows());
    let style = Style::new(Face::ArchivoExtraBold, SIZE_CUSTOMER).tracked(TRACK_CUSTOMER);
    let mut dropped = 0;

    for treatment in MarkerTreatment::ALL {
        let settings = Settings {
            marker: treatment,
            ..Settings::default()
        };

        for order in &orders {
            let (page, _) = stop::block(order, &settings, &Cursor::new(0.0));
            let where_ = format!("{} under {:?}", order.customer, treatment);

            let name = page
                .primitives
                .iter()
                .find_map(|primitive| match primitive {
                    Primitive::Text {
                        baseline_start,
                        text,
                        ..
                    } if *text == order.customer => Some(*baseline_start),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{where_}: the customer name is on the block"));
            let name_right = name.x + text::width(&order.customer, style);

            let (marker_text, loud) = page
                .primitives
                .iter()
                .find_map(|primitive| match primitive {
                    Primitive::Text {
                        baseline_start,
                        text,
                        ..
                    } if text.starts_with("want substitute") => Some((*baseline_start, false)),
                    Primitive::Text {
                        baseline_start,
                        text,
                        ..
                    } if text.starts_with("WANT SUBSTITUTE") => Some((*baseline_start, true)),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{where_}: the marker is on the block"));
            let marker_left = marker_text.x
                - if loud && treatment.has_badge() {
                    BADGE_PADDING.1
                } else {
                    0.0
                };

            assert!(
                name_right <= marker_left + 0.01,
                "{where_}: the name ends at {name_right:.2} and the marker starts at \
                 {marker_left:.2}"
            );

            let department = page
                .primitives
                .iter()
                .find_map(|primitive| match primitive {
                    Primitive::Box {
                        rect,
                        stroke: Some(Stroke { weight, .. }),
                        ..
                    } if (*weight - RULE_DEPARTMENT_BOX).abs() < 0.01 => Some(*rect),
                    _ => None,
                });
            assert_eq!(
                department.is_some(),
                order.department.is_some(),
                "{where_}: the DPT box appears exactly when there is a department"
            );
            if let Some(box_rect) = department {
                assert!(
                    box_rect.y > name.y,
                    "{where_}: the DPT box belongs below the name's baseline"
                );
            }

            let glyphs: Vec<_> = page
                .primitives
                .iter()
                .filter_map(|primitive| match primitive {
                    Primitive::Box { rect, .. } if (rect.width - CRATE_GLYPH.0).abs() < 0.01 => {
                        Some(*rect)
                    }
                    _ => None,
                })
                .collect();
            if treatment == MarkerTreatment::default()
                && glyphs.iter().any(|glyph| glyph.y >= name.y)
            {
                dropped += 1;
            }

            for glyph in glyphs {
                if glyph.y < name.y {
                    assert!(
                        glyph.x >= name_right + CRATE_GAP - 0.01,
                        "{where_}: a crate at {:.2} overlaps a name ending at {name_right:.2}",
                        glyph.x
                    );
                    assert!(
                        glyph.right() <= marker_left - MARKER_GAP + 0.01,
                        "{where_}: a crate ends at {:.2}, past the marker at {marker_left:.2}",
                        glyph.right()
                    );
                } else if let Some(box_rect) = department {
                    assert!(
                        glyph.x >= box_rect.right(),
                        "{where_}: a crate at {:.2} overlaps the DPT box ending at {:.2}",
                        glyph.x,
                        box_rect.right()
                    );
                }
            }
        }
    }

    assert_eq!(
        dropped, 5,
        "five sample names are long enough to push their crates down a line"
    );
}
