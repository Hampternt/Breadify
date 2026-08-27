//! Drawing a display list on screen.
//!
//! The second renderer, and the reason the display list exists: it draws what
//! the layout already decided and measures nothing, so what the preview shows
//! and what the printer prints cannot drift apart.

use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};

use super::theme;
use crate::geometry::{Mm, PAGE_HEIGHT, PAGE_WIDTH, pt_to_mm};
use crate::page::{Colour, Page, Primitive};

/// Draws `page` with its top-left corner at `origin`, `scale` pixels to the
/// millimetre.
///
/// Letter-spacing is not applied on screen — egui sets a run in one call and
/// the difference is under a fifth of a millimetre at these sizes. Nothing is
/// positioned from it; the paper is the authority.
pub fn draw(painter: &egui::Painter, page: &Page, origin: Pos2, scale: f32) {
    for primitive in &page.primitives {
        match primitive {
            Primitive::Text {
                baseline_start,
                text,
                style,
                colour,
            } => {
                // egui anchors text by its bounding box rather than by the
                // font's baseline, so a preview sits a hair high; nothing is
                // positioned from it, and the paper is the authority.
                let size = pt_to_mm(style.size) as f32 * scale;
                painter.text(
                    origin + Vec2::new(baseline_start.x as f32, baseline_start.y as f32) * scale,
                    egui::Align2::LEFT_BOTTOM,
                    text,
                    egui::FontId::new(size, theme::family_for(style.face)),
                    ink(*colour),
                );
            }

            Primitive::Rule {
                from,
                to,
                weight,
                colour,
            } => {
                painter.line_segment(
                    [
                        origin + Vec2::new(from.x as f32, from.y as f32) * scale,
                        origin + Vec2::new(to.x as f32, to.y as f32) * scale,
                    ],
                    Stroke::new(thickness(*weight, scale), ink(*colour)),
                );
            }

            Primitive::Box {
                rect,
                fill,
                stroke,
                radius,
            } => {
                let on_screen = Rect::from_min_size(
                    origin + Vec2::new(rect.x as f32, rect.y as f32) * scale,
                    Vec2::new(rect.width as f32, rect.height as f32) * scale,
                );
                let corner =
                    CornerRadius::same((*radius as f32 * scale).round().clamp(0.0, 255.0) as u8);

                if let Some(colour) = fill {
                    painter.rect_filled(on_screen, corner, ink(*colour));
                }
                if let Some(stroke) = stroke {
                    painter.rect_stroke(
                        on_screen,
                        corner,
                        Stroke::new(thickness(stroke.weight, scale), ink(stroke.colour)),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
    }
}

/// Paints the sheet itself — white paper under the ink.
pub fn paper(painter: &egui::Painter, origin: Pos2, scale: f32, height: Mm) {
    let sheet = Rect::from_min_size(origin, Vec2::new(PAGE_WIDTH as f32, height as f32) * scale);
    painter.rect_filled(sheet, CornerRadius::same(2), Color32::WHITE);
}

/// A whole page, scaled to fit a box.
pub fn scale_to_fit(available: Vec2) -> f32 {
    (available.x / PAGE_WIDTH as f32).min(available.y / PAGE_HEIGHT as f32)
}

/// A rule weight in points, as pixels — never thinner than a hairline, or it
/// disappears at preview scale.
fn thickness(weight: crate::geometry::Pt, scale: f32) -> f32 {
    (pt_to_mm(weight) as f32 * scale).max(0.7)
}

fn ink(colour: Colour) -> Color32 {
    Color32::from_rgb(colour.red, colour.green, colour.blue)
}
