//! Reading the mark out of its SVG.

use breadify::artwork;
use breadify::page::Colour;

#[test]
fn the_mark_reads_as_eighteen_filled_paths() {
    let mark = artwork::wordmark();

    assert_eq!((mark.width, mark.height), (166.0, 40.0));
    assert_eq!(mark.shapes.len(), 18);
}

#[test]
fn it_is_a_red_bag_and_white_type() {
    let mark = artwork::wordmark();
    let red = Colour::rgb(0xFF, 0x4F, 0x46);
    let white = Colour::grey(0xFF);

    let colours: std::collections::BTreeSet<(u8, u8, u8)> = mark
        .shapes
        .iter()
        .map(|shape| (shape.fill.red, shape.fill.green, shape.fill.blue))
        .collect();

    assert_eq!(
        colours,
        [
            (red.red, red.green, red.blue),
            (white.red, white.green, white.blue)
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(mark.shapes[0].fill, red, "the bag is drawn first");
}

#[test]
fn every_outline_is_closed_and_inside_the_artwork() {
    let mark = artwork::wordmark();

    for (index, shape) in mark.shapes.iter().enumerate() {
        assert!(!shape.rings.is_empty(), "shape {index} has no outline");
        for ring in &shape.rings {
            assert!(
                ring.len() >= 3,
                "shape {index} has an outline of {} points",
                ring.len()
            );
            for vertex in ring {
                assert!(
                    (-1.0..=mark.width + 1.0).contains(&vertex.x)
                        && (-1.0..=mark.height + 1.0).contains(&vertex.y),
                    "shape {index} leaves the artwork at ({}, {})",
                    vertex.x,
                    vertex.y
                );
            }
        }
    }
}

#[test]
fn the_letters_with_holes_have_more_than_one_outline() {
    let mark = artwork::wordmark();
    let with_holes = mark
        .shapes
        .iter()
        .filter(|shape| shape.rings.len() > 1)
        .count();

    assert!(
        with_holes > 0,
        "a, e, o and p all have counters — none survived"
    );
}

#[test]
fn curves_carry_their_control_points() {
    let mark = artwork::wordmark();
    let controls: usize = mark
        .shapes
        .iter()
        .flat_map(|shape| shape.rings.iter())
        .flat_map(|ring| ring.iter())
        .filter(|vertex| vertex.control)
        .count();

    assert!(controls > 100, "a mark this round needs its beziers");
}

/// The window icon is the mark's symbol and nothing else, rasterised. The
/// wordmark's lettering would be a stripe in a square tile.
#[test]
fn the_window_icon_is_the_symbol_on_a_transparent_tile() {
    let (rgba, edge) = breadify::icon::window_icon();
    assert_eq!(rgba.len(), (edge * edge * 4) as usize);

    let pixel = |x: u32, y: u32| {
        let at = ((y * edge + x) * 4) as usize;
        (rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3])
    };

    for (x, y) in [(0, 0), (edge - 1, 0), (0, edge - 1), (edge - 1, edge - 1)] {
        assert_eq!(pixel(x, y).3, 0, "the tile's corners are paper, not bag");
    }

    let brand = breadify::page::BRAND_RED;
    let red = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] == 0xFF && (p[0], p[1], p[2]) == (brand.red, brand.green, brand.blue))
        .count();
    let white = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] == 0xFF && (p[0], p[1], p[2]) == (0xFF, 0xFF, 0xFF))
        .count();

    assert!(red > white, "the bag is the ground, the smile is on it");
    assert!(
        white > 400,
        "the smile survives at {edge} px: {white} pixels"
    );
    assert!(
        red + white > ((edge * edge) / 3) as usize,
        "the symbol fills the tile it was fitted to"
    );
}
