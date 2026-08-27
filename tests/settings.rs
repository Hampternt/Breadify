//! Every control on the Configure step has to move the paper. A toggle nobody
//! wired up is worse than no toggle at all.

mod support;

use breadify::crates::{self, CrateRules};
use breadify::layout::{Cursor, MarkerTreatment, Settings, stop};
use breadify::order::{self, Order};
use breadify::page::{Page, Primitive};
use support::sample_rows;

fn stop_that_refuses() -> Order {
    order::fold(&sample_rows())
        .into_iter()
        .find(|order| !order.accept_alternatives && order.lines.len() > 1)
        .expect("some order refuses substitutes")
}

fn block(settings: &Settings) -> Page {
    stop::block(&stop_that_refuses(), settings, &Cursor::new(0.0)).0
}

fn runs(page: &Page) -> Vec<String> {
    page.primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

fn heavy_rules(page: &Page) -> usize {
    page.primitives
        .iter()
        .filter(|primitive| matches!(primitive, Primitive::Rule { weight, .. } if *weight >= 5.0))
        .count()
}

fn filled_boxes(page: &Page) -> usize {
    page.primitives
        .iter()
        .filter(|primitive| matches!(primitive, Primitive::Box { fill: Some(_), .. }))
        .count()
}

#[test]
fn hiding_the_order_id_takes_it_off_the_page() {
    let stop = stop_that_refuses();
    let shown = Settings::default();
    let hidden = Settings {
        show_order_id: false,
        ..Settings::default()
    };

    assert!(runs(&block(&shown)).contains(&stop.id.to_string()));
    assert!(!runs(&block(&hidden)).contains(&stop.id.to_string()));
}

#[test]
fn each_marker_treatment_draws_what_it_says() {
    let badge = Settings {
        marker: MarkerTreatment::InvertedBadge,
        ..Settings::default()
    };
    let rule_only = Settings {
        marker: MarkerTreatment::HeavyRule,
        ..Settings::default()
    };
    let words_only = Settings {
        marker: MarkerTreatment::WordOnly,
        ..Settings::default()
    };

    // The words are there whatever the treatment.
    for settings in [&badge, &rule_only, &words_only] {
        assert!(runs(&block(settings)).contains(&"WANT SUBSTITUTE: FALSE".to_owned()));
    }

    assert_eq!(heavy_rules(&block(&badge)), 1, "badge keeps the bar");
    assert_eq!(heavy_rules(&block(&rule_only)), 1, "the bar alone");
    assert_eq!(heavy_rules(&block(&words_only)), 0, "no bar");

    assert!(
        filled_boxes(&block(&badge)) > filled_boxes(&block(&rule_only)),
        "only the badge fills a box behind the words"
    );
}

#[test]
fn a_bread_that_takes_more_room_needs_more_crates() {
    let stop = stop_that_refuses();
    let plain = Settings::default();
    let before = crates::count(&stop, &plain.crates).total();

    let mut bulky = CrateRules::default();
    for line in &stop.lines {
        bulky.set_size(line.product.id, 300);
    }
    let after = crates::count(&stop, &bulky).total();

    assert!(
        after > before,
        "tripling every bread's size should need more crates: {before} -> {after}"
    );
}

#[test]
fn crate_capacity_changes_the_glyphs_on_the_page() {
    let stop = stop_that_refuses();
    let default = Settings::default();
    let small_crates = Settings {
        crates: CrateRules {
            large_capacity: 4,
            small_capacity: 2,
            ..CrateRules::default()
        },
        ..Settings::default()
    };

    let glyphs = |settings: &Settings| {
        stop::block(&stop, settings, &Cursor::new(0.0))
            .0
            .primitives
            .iter()
            .filter(|primitive| {
                matches!(primitive, Primitive::Box { rect, .. }
                    if (rect.width - 7.1).abs() < 0.01)
            })
            .count()
    };

    assert!(
        glyphs(&small_crates) > glyphs(&default),
        "smaller crates, more of them"
    );
}
