//! The bread guy, sitting behind the Check step at almost no opacity.
//!
//! He is a joke and nothing else — no state, no interaction, nothing under
//! him that a person needs to read through him. He decodes on first use and is
//! uploaded once; if the decode ever fails he simply is not drawn, and the
//! step still reads, though its finding cards keep the translucent fill they
//! were given for his sake (`theme::CARD_VEIL`).

use std::sync::LazyLock;

use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2, pos2};

use super::Breadify;

const JPEG: &[u8] = include_bytes!("../../assets/breadguy.jpg");

/// How much of the step he is allowed to cover.
const FILL: f32 = 0.78;

/// How far from invisible. High enough to be seen on purpose rather than
/// noticed by accident, low enough that the findings still win the page.
const ALPHA: u8 = 34;

/// The decoded pixels and their size, worked out once.
static PIXELS: LazyLock<Option<(Vec<u8>, [usize; 2])>> = LazyLock::new(decode);

fn decode() -> Option<(Vec<u8>, [usize; 2])> {
    let decoded = image::load_from_memory_with_format(JPEG, image::ImageFormat::Jpeg).ok()?;
    let rgba = decoded.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some((rgba.into_raw(), size))
}

/// The decoded mascot, for a test that the asset in the repo is still a
/// picture and not, say, a text file someone renamed.
pub fn decoded() -> Option<(&'static [u8], [usize; 2])> {
    PIXELS
        .as_ref()
        .map(|(pixels, size)| (pixels.as_slice(), *size))
}

/// Paints him behind whatever the caller draws next — the painter writes in
/// call order, so everything the step draws after this lands on top.
pub fn behind(app: &mut Breadify, ui: &egui::Ui) {
    let Some(texture) = uploaded(app, ui.ctx()) else {
        return;
    };

    let area = ui.max_rect();
    if !area.height().is_finite() || area.height() < 1.0 || area.width() < 1.0 {
        return;
    }

    let size = texture.size_vec2();
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }

    let scale = (area.height() * FILL / size.y).min(area.width() * FILL / size.x);
    let rect = Rect::from_center_size(area.center(), Vec2::new(size.x, size.y) * scale);

    ui.painter().image(
        texture.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::from_white_alpha(ALPHA),
    );
}

/// The texture, uploaded on the first frame that asks for it and kept for the
/// life of the window.
fn uploaded(app: &mut Breadify, context: &egui::Context) -> Option<TextureHandle> {
    if app.mascot.is_none() {
        let (pixels, size) = PIXELS.as_ref()?;
        app.mascot = Some(context.load_texture(
            "breadguy",
            ColorImage::from_rgba_unmultiplied(*size, pixels),
            TextureOptions::LINEAR,
        ));
    }
    app.mascot.clone()
}
