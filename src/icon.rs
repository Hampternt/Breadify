//! The window icon: the Matvare Expressen symbol, rasterised.
//!
//! The paper wants vector and gets it; a window manager wants pixels. The
//! symbol — the red bag, the first two paths of the mark — is squarish, so it
//! survives being cut down to a taskbar tile where the full wordmark would
//! turn into a stripe.
//!
//! Rendering here rather than shipping a PNG keeps one source of truth for the
//! mark, and costs a scanline fill over about two hundred edges at startup.

use crate::artwork::{self, Artwork, Vertex};

/// The icon's edge, in pixels. Enough for a Windows taskbar and a GNOME dock
/// without either one having to invent detail.
const SIZE: u32 = 256;

/// How many samples across a pixel. The fill itself is hard-edged; the
/// antialiasing is the box filter on the way down.
const SUPERSAMPLE: u32 = 4;

/// Segments a cubic is flattened into. At 1024 px across the whole mark, a
/// curve spans well under sixteen pixels.
const CURVE_STEPS: usize = 12;

/// Everything left of this in the artwork's own coordinates is the symbol;
/// everything right of it is the wordmark's lettering.
const SYMBOL_RIGHT: f64 = 41.0;

/// The share of the tile left empty around the symbol.
const PADDING: f64 = 0.06;

/// The icon as straight RGBA rows, top row first, and its edge in pixels.
///
/// Transparent where the symbol is not: the bag is its own silhouette, so a
/// dark dock and a light one both get the shape they expect.
pub fn window_icon() -> (Vec<u8>, u32) {
    (render(artwork::wordmark(), SIZE * SUPERSAMPLE), SIZE)
}

/// One flattened outline in pixel space.
type Outline = Vec<(f64, f64)>;

fn render(art: &Artwork, high: u32) -> Vec<u8> {
    let symbol: Vec<&crate::artwork::Shape> = art
        .shapes
        .iter()
        .filter(|shape| {
            shape
                .rings
                .iter()
                .flatten()
                .all(|vertex| vertex.x <= SYMBOL_RIGHT)
        })
        .collect();

    let Some(bounds) = bounds(&symbol) else {
        return vec![0; (SIZE * SIZE * 4) as usize];
    };

    let mut canvas = vec![0u8; (high * high * 4) as usize];
    for shape in symbol {
        let outlines: Vec<Outline> = shape
            .rings
            .iter()
            .map(|ring| place(&flatten(ring), bounds, high))
            .collect();
        fill(&mut canvas, high, &outlines, shape.fill);
    }
    downsample(&canvas, high)
}

/// The symbol's own box: `(left, top, edge)`, square so the aspect survives.
fn bounds(shapes: &[&crate::artwork::Shape]) -> Option<(f64, f64, f64)> {
    let points: Vec<&Vertex> = shapes
        .iter()
        .flat_map(|shape| shape.rings.iter().flatten())
        .collect();
    let first = points.first()?;

    let mut left = first.x;
    let mut right = first.x;
    let mut top = first.y;
    let mut bottom = first.y;
    for point in &points {
        left = left.min(point.x);
        right = right.max(point.x);
        top = top.min(point.y);
        bottom = bottom.max(point.y);
    }

    let edge = (right - left).max(bottom - top);
    Some((
        (left + right) / 2.0 - edge / 2.0,
        (top + bottom) / 2.0 - edge / 2.0,
        edge,
    ))
}

/// Turns one ring into a polygon, replacing every cubic with straight
/// segments. Control points come in pairs, each pair followed by the point the
/// curve lands on.
fn flatten(ring: &[Vertex]) -> Outline {
    let mut out: Outline = Vec::new();
    let mut index = 0;

    while index < ring.len() {
        if !ring[index].control {
            out.push((ring[index].x, ring[index].y));
            index += 1;
            continue;
        }

        let start = *out.last().unwrap_or(&(ring[index].x, ring[index].y));
        let first = (ring[index].x, ring[index].y);
        let Some(second) = ring.get(index + 1).map(|v| (v.x, v.y)) else {
            break;
        };
        let end = ring
            .get(index + 2)
            .map_or_else(|| out.first().copied().unwrap_or(start), |v| (v.x, v.y));

        for step in 1..=CURVE_STEPS {
            let t = step as f64 / CURVE_STEPS as f64;
            out.push(cubic(start, first, second, end, t));
        }
        index += 3;
    }
    out
}

fn cubic(
    start: (f64, f64),
    first: (f64, f64),
    second: (f64, f64),
    end: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let inverse = 1.0 - t;
    let weights = [
        inverse * inverse * inverse,
        3.0 * inverse * inverse * t,
        3.0 * inverse * t * t,
        t * t * t,
    ];
    (
        weights[0] * start.0 + weights[1] * first.0 + weights[2] * second.0 + weights[3] * end.0,
        weights[0] * start.1 + weights[1] * first.1 + weights[2] * second.1 + weights[3] * end.1,
    )
}

/// Moves an outline from the artwork's coordinates into the tile's.
fn place(outline: &Outline, bounds: (f64, f64, f64), high: u32) -> Outline {
    let (left, top, edge) = bounds;
    let inset = f64::from(high) * PADDING;
    let scale = (f64::from(high) - 2.0 * inset) / edge;
    outline
        .iter()
        .map(|(x, y)| (inset + (x - left) * scale, inset + (y - top) * scale))
        .collect()
}

/// Scanline fill, non-zero winding — the rule SVG fills by, and the one that
/// puts a hole in a counter without needing to know which ring is the hole.
fn fill(canvas: &mut [u8], high: u32, outlines: &[Outline], colour: crate::page::Colour) {
    let edges: Vec<((f64, f64), (f64, f64))> = outlines
        .iter()
        .filter(|outline| outline.len() > 2)
        .flat_map(|outline| {
            outline
                .iter()
                .zip(outline.iter().cycle().skip(1))
                .map(|(from, to)| (*from, *to))
                .take(outline.len())
        })
        .filter(|(from, to)| (from.1 - to.1).abs() > f64::EPSILON)
        .collect();

    let mut crossings: Vec<(f64, i32)> = Vec::new();
    for row in 0..high {
        let y = f64::from(row) + 0.5;
        crossings.clear();
        crossings.extend(edges.iter().filter_map(|(from, to)| {
            if (from.1 <= y) == (to.1 <= y) {
                return None;
            }
            let along = (y - from.1) / (to.1 - from.1);
            Some((
                from.0 + along * (to.0 - from.0),
                if to.1 > from.1 { 1 } else { -1 },
            ))
        }));
        crossings.sort_by(|left, right| left.0.total_cmp(&right.0));

        let mut winding = 0;
        for pair in crossings.windows(2) {
            winding += pair[0].1;
            if winding == 0 {
                continue;
            }
            let from = pair[0].0.max(0.0).round() as u32;
            let to = pair[1].0.min(f64::from(high)).round() as u32;
            for column in from..to.min(high) {
                let at = ((row * high + column) * 4) as usize;
                canvas[at] = colour.red;
                canvas[at + 1] = colour.green;
                canvas[at + 2] = colour.blue;
                canvas[at + 3] = 0xFF;
            }
        }
    }
}

/// Box-filters the oversized canvas down to the icon, averaging only the
/// covered samples so an edge pixel keeps its colour and loses its opacity
/// instead of fading towards black.
fn downsample(canvas: &[u8], high: u32) -> Vec<u8> {
    let factor = high / SIZE;
    let per_pixel = factor * factor;
    let mut icon = vec![0u8; (SIZE * SIZE * 4) as usize];

    for row in 0..SIZE {
        for column in 0..SIZE {
            let (mut red, mut green, mut blue, mut covered) = (0u32, 0u32, 0u32, 0u32);
            for y in 0..factor {
                for x in 0..factor {
                    let at = (((row * factor + y) * high + column * factor + x) * 4) as usize;
                    if canvas[at + 3] == 0 {
                        continue;
                    }
                    red += u32::from(canvas[at]);
                    green += u32::from(canvas[at + 1]);
                    blue += u32::from(canvas[at + 2]);
                    covered += 1;
                }
            }

            let at = ((row * SIZE + column) * 4) as usize;
            if covered == 0 {
                continue;
            }
            icon[at] = (red / covered) as u8;
            icon[at + 1] = (green / covered) as u8;
            icon[at + 2] = (blue / covered) as u8;
            icon[at + 3] = (covered * 255 / per_pixel) as u8;
        }
    }
    icon
}
