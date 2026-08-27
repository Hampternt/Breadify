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
