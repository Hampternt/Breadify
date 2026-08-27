//! Drawing a display list to PDF.
//!
//! The only thing this module decides is where the origin is: the layout
//! measures from the top-left corner of the sheet, PDF measures from the
//! bottom-left. Everything else is already positioned by the time it gets
//! here — no measuring, no layout, no second opinion about the page.

use std::collections::HashMap;
use std::path::Path;

use printpdf::{
    Color, FontId, Line, LinePoint, Op, PaintMode, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point as PdfPoint, Polygon, PolygonRing, Pt as PdfPt, Rgb, TextItem,
    WindingOrder,
};

use crate::font::{ALL, Face};
use crate::geometry::{Mm, PAGE_HEIGHT, PAGE_WIDTH, Point, mm_to_pt};
use crate::page::{Colour, Page, Primitive, Stroke};

/// Renders pages to PDF bytes.
///
/// # Errors
///
/// Fails only if an embedded font cannot be parsed, which would be a broken
/// build rather than a runtime condition.
pub fn render(pages: &[Page], title: &str) -> Result<Vec<u8>, RenderError> {
    let mut document = PdfDocument::new(title);
    let mut fonts: HashMap<Face, FontId> = HashMap::new();

    for face in ALL {
        let mut warnings = Vec::new();
        let parsed = printpdf::ParsedFont::from_bytes(face.bytes(), 0, &mut warnings)
            .ok_or(RenderError::Font { face })?;
        fonts.insert(face, document.add_font(&parsed));
    }

    let drawn: Vec<PdfPage> = pages
        .iter()
        .map(|page| {
            PdfPage::new(
                printpdf::Mm(PAGE_WIDTH as f32),
                printpdf::Mm(PAGE_HEIGHT as f32),
                operations(page, &fonts),
            )
        })
        .collect();

    let mut warnings = Vec::new();
    Ok(document
        .with_pages(drawn)
        .save(&PdfSaveOptions::default(), &mut warnings))
}

/// Renders pages and writes them to `path`.
///
/// # Errors
///
/// Fails if a font cannot be parsed or the file cannot be written.
pub fn write(path: &Path, pages: &[Page], title: &str) -> Result<(), RenderError> {
    let bytes = render(pages, title)?;
    std::fs::write(path, bytes).map_err(|source| RenderError::Write {
        path: path.display().to_string(),
        source,
    })
}

/// What can go wrong on the way to a file.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("the embedded font {face:?} could not be parsed")]
    Font { face: Face },

    #[error("could not write {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Turns one page's primitives into PDF operations, in order.
fn operations(page: &Page, fonts: &HashMap<Face, FontId>) -> Vec<Op> {
    let mut ops = Vec::with_capacity(page.primitives.len() * 4);

    for primitive in &page.primitives {
        match primitive {
            Primitive::Text {
                baseline_start,
                text,
                style,
                colour,
            } => {
                let Some(font) = fonts.get(&style.face) else {
                    continue;
                };
                ops.push(Op::StartTextSection);
                ops.push(Op::SetFillColor { col: ink(*colour) });
                ops.push(Op::SetFont {
                    font: PdfFontHandle::External(font.clone()),
                    size: PdfPt(style.size as f32),
                });
                ops.push(Op::SetCharacterSpacing {
                    multiplier: (style.tracking_em * style.size) as f32,
                });
                ops.push(Op::SetTextCursor {
                    pos: point(*baseline_start),
                });
                ops.push(Op::ShowText {
                    items: vec![TextItem::Text(text.clone())],
                });
                ops.push(Op::EndTextSection);
            }

            Primitive::Rule {
                from,
                to,
                weight,
                colour,
            } => {
                ops.push(Op::SetOutlineColor { col: ink(*colour) });
                ops.push(Op::SetOutlineThickness {
                    pt: PdfPt(*weight as f32),
                });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: point(*from),
                                bezier: false,
                            },
                            LinePoint {
                                p: point(*to),
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });
            }

            Primitive::Box {
                rect,
                fill,
                stroke,
                radius,
            } => {
                if let Some(colour) = fill {
                    ops.push(Op::SetFillColor { col: ink(*colour) });
                }
                if let Some(Stroke { weight, colour }) = stroke {
                    ops.push(Op::SetOutlineColor { col: ink(*colour) });
                    ops.push(Op::SetOutlineThickness {
                        pt: PdfPt(*weight as f32),
                    });
                }
                let mode = paint_mode(fill.is_some(), stroke.is_some());
                if *radius > 0.0 {
                    ops.push(Op::DrawPolygon {
                        polygon: rounded(*rect, *radius, mode),
                    });
                    continue;
                }
                ops.push(Op::DrawRectangle {
                    rectangle: printpdf::Rect {
                        x: PdfPt(mm_to_pt(rect.x) as f32),
                        y: PdfPt(mm_to_pt(PAGE_HEIGHT - rect.bottom()) as f32),
                        width: PdfPt(mm_to_pt(rect.width) as f32),
                        height: PdfPt(mm_to_pt(rect.height) as f32),
                        mode: Some(mode),
                        winding_order: None,
                    },
                });
            }
        }
    }

    ops
}

/// A box with rounded corners, as a polygon whose corner points are bezier
/// control points — the tick boxes and crate glyphs, which the design gives a
/// radius of a third of a millimetre.
fn rounded(rect: crate::geometry::Rect, radius: Mm, mode: PaintMode) -> Polygon {
    let radius = radius.min(rect.width / 2.0).min(rect.height / 2.0);
    let (left, right) = (rect.x, rect.right());
    let (top, bottom) = (flip(rect.y), flip(rect.bottom()));

    let corners = [
        // Along the bottom edge, up the right, back along the top, down the left.
        ((left + radius, bottom), false),
        ((right - radius, bottom), false),
        ((right, bottom), true),
        ((right, bottom + radius), false),
        ((right, top - radius), false),
        ((right, top), true),
        ((right - radius, top), false),
        ((left + radius, top), false),
        ((left, top), true),
        ((left, top - radius), false),
        ((left, bottom + radius), false),
        ((left, bottom), true),
    ];

    Polygon {
        rings: vec![PolygonRing {
            points: corners
                .into_iter()
                .map(|((x, y), bezier)| LinePoint {
                    p: PdfPoint {
                        x: PdfPt(mm_to_pt(x) as f32),
                        y: PdfPt(mm_to_pt(y) as f32),
                    },
                    bezier,
                })
                .collect(),
        }],
        mode,
        winding_order: WindingOrder::NonZero,
    }
}

/// A layout point, in PDF's own space: points, from the bottom-left corner.
fn point(at: Point) -> PdfPoint {
    PdfPoint {
        x: PdfPt(mm_to_pt(at.x) as f32),
        y: PdfPt(mm_to_pt(flip(at.y)) as f32),
    }
}

/// Turns a distance down the page into a distance up from its foot.
fn flip(y: Mm) -> Mm {
    PAGE_HEIGHT - y
}

fn paint_mode(filled: bool, stroked: bool) -> PaintMode {
    match (filled, stroked) {
        (true, true) => PaintMode::FillStroke,
        (true, false) => PaintMode::Fill,
        _ => PaintMode::Stroke,
    }
}

/// One of the design's inks, as PDF wants it.
fn ink(colour: Colour) -> Color {
    Color::Rgb(Rgb {
        r: f32::from(colour.red) / 255.0,
        g: f32::from(colour.green) / 255.0,
        b: f32::from(colour.blue) / 255.0,
        icc_profile: None,
    })
}
