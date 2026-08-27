//! Two jokes, sitting behind two steps at almost no opacity.
//!
//! They are jokes and nothing else — no state, no interaction, nothing under
//! them that a person needs to read through them. Each decodes on first use
//! and is uploaded once; if a decode ever fails it simply is not drawn.
//!
//! One caveat: the Check step's finding cards were given a translucent fill
//! (`theme::CARD_VEIL`) for the bread guy's sake, and `check::card` has no way
//! to ask whether he made it. A decode failure there leaves the cards
//! translucent over a flat page rather than restoring the opaque ones.

use std::sync::LazyLock;

use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2, pos2};

use super::Breadify;

/// Which joke, and where it belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mascot {
    /// Behind the Check step: a bread roll at a computer in a Matvare
    /// Expressen cap, sweating.
    BreadGuy,
    /// Behind the Open step, and only while nothing has been opened — which is
    /// the one moment in the day when there is, in fact, no bread.
    NoBread,
}

/// Every mascot, for tests.
pub const ALL: [Mascot; 2] = [Mascot::BreadGuy, Mascot::NoBread];

impl Mascot {
    fn jpeg(self) -> &'static [u8] {
        match self {
            Self::BreadGuy => include_bytes!("../../assets/breadmve.jpg"),
            Self::NoBread => include_bytes!("../../assets/nobread.jpg"),
        }
    }

    /// How much of the step it is allowed to cover.
    fn fill(self) -> f32 {
        match self {
            Self::BreadGuy => 0.78,
            // The drop zone's copy runs across its middle, so this one keeps
            // to the room above and below it.
            Self::NoBread => 0.92,
        }
    }

    /// How far from invisible. High enough to be seen on purpose rather than
    /// noticed by accident, low enough that the step still wins the page.
    fn alpha(self) -> u8 {
        match self {
            Self::BreadGuy => 34,
            // Higher: this one is a mid-grey photograph on a near-black
            // panel, where the bread guy is a bright one behind opaque cards.
            // At the bread guy's 34 the face disappears and only the caption
            // survives, which is the half of the joke that needs the other.
            Self::NoBread => 52,
        }
    }

    /// How far up the step to nudge it, as a share of the step's height.
    ///
    /// Centred, Megamind's eyes land squarely behind the drop zone's heading —
    /// which is the half of that picture worth seeing. Up a little puts them
    /// in the empty room above it and the caption below the button.
    fn lift(self) -> f32 {
        match self {
            Self::BreadGuy => 0.0,
            Self::NoBread => 0.10,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::BreadGuy => 0,
            Self::NoBread => 1,
        }
    }

    fn pixels(self) -> Option<&'static (Vec<u8>, [usize; 2])> {
        match self {
            Self::BreadGuy => BREAD_GUY.as_ref(),
            Self::NoBread => NO_BREAD.as_ref(),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::BreadGuy => "breadmve",
            Self::NoBread => "nobread",
        }
    }
}

/// The decoded pixels, worked out once each.
static BREAD_GUY: LazyLock<Option<(Vec<u8>, [usize; 2])>> =
    LazyLock::new(|| decode(Mascot::BreadGuy));
static NO_BREAD: LazyLock<Option<(Vec<u8>, [usize; 2])>> =
    LazyLock::new(|| decode(Mascot::NoBread));

fn decode(who: Mascot) -> Option<(Vec<u8>, [usize; 2])> {
    let decoded = image::load_from_memory_with_format(who.jpeg(), image::ImageFormat::Jpeg).ok()?;
    let rgba = decoded.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some((rgba.into_raw(), size))
}

/// A decoded mascot, for a test that the asset in the repo is still a picture
/// and not, say, a text file someone renamed.
pub fn decoded(who: Mascot) -> Option<(&'static [u8], [usize; 2])> {
    who.pixels()
        .map(|(pixels, size)| (pixels.as_slice(), *size))
}

/// Paints one behind whatever the caller draws next, fitted into `area` — the
/// painter writes in call order, so everything drawn after this lands on top.
pub fn behind(app: &mut Breadify, ui: &egui::Ui, area: Rect, who: Mascot) {
    let Some(texture) = uploaded(app, ui.ctx(), who) else {
        return;
    };
    if !area.height().is_finite() || area.height() < 1.0 || area.width() < 1.0 {
        return;
    }

    let size = texture.size_vec2();
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }

    let scale = (area.height() * who.fill() / size.y).min(area.width() * who.fill() / size.x);
    let centre = area.center() - Vec2::new(0.0, area.height() * who.lift());
    let rect = Rect::from_center_size(centre, Vec2::new(size.x, size.y) * scale);

    ui.painter_at(area).image(
        texture.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::from_white_alpha(who.alpha()),
    );
}

/// The texture, uploaded on the first frame that asks for it and kept for the
/// life of the window.
fn uploaded(app: &mut Breadify, context: &egui::Context, who: Mascot) -> Option<TextureHandle> {
    let slot = who.index();
    if app.mascots[slot].is_none() {
        let (pixels, size) = who.pixels()?;
        app.mascots[slot] = Some(context.load_texture(
            who.name(),
            ColorImage::from_rgba_unmultiplied(*size, pixels),
            TextureOptions::LINEAR,
        ));
    }
    app.mascots[slot].clone()
}
