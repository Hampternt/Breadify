//! The screen half of the design system: the dark palette, the spacing scale
//! and the typefaces, as named values rather than literals in the widgets.

use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke};

use crate::font::Face;

/// Backgrounds, from the window's ground up.
pub const PAGE: Color32 = Color32::from_rgb(0x0B, 0x09, 0x10);
pub const VOID: Color32 = Color32::from_rgb(0x07, 0x06, 0x0B);
pub const RAISED: Color32 = Color32::from_rgb(0x0E, 0x0C, 0x14);
pub const CARD: Color32 = Color32::from_rgb(0x17, 0x14, 0x1F);
pub const CHIP: Color32 = Color32::from_rgb(0x26, 0x22, 0x32);

/// Text, from loudest to faintest.
pub const STRONG: Color32 = Color32::from_rgb(0xF2, 0xEE, 0xF8);
pub const BODY: Color32 = Color32::from_rgb(0xCD, 0xC6, 0xDD);
pub const MUTED: Color32 = Color32::from_rgb(0x8D, 0x87, 0xA0);
pub const FAINT: Color32 = Color32::from_rgb(0x5F, 0x58, 0x76);

/// The accent, and the tints built from it.
pub const ACCENT: Color32 = Color32::from_rgb(0xB4, 0x8E, 0xF7);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xCB, 0xB0, 0xFF);
pub const ACCENT_PRESS: Color32 = Color32::from_rgb(0x8F, 0x6F, 0xD8);
pub const ACCENT_TINT: Color32 = Color32::from_rgba_premultiplied(0x19, 0x14, 0x22, 0x24);
pub const ACCENT_BORDER: Color32 = Color32::from_rgba_premultiplied(0x51, 0x40, 0x6F, 0x73);

/// Borders.
pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(0x11, 0x11, 0x12, 0x12);
pub const BORDER: Color32 = Color32::from_rgb(0x26, 0x22, 0x32);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x32, 0x2C, 0x42);

/// Status.
pub const SUCCESS: Color32 = Color32::from_rgb(0x4F, 0xD6, 0xA8);
pub const WARNING: Color32 = Color32::from_rgb(0xFF, 0xB5, 0x70);
pub const DANGER: Color32 = Color32::from_rgb(0xF7, 0x76, 0x8E);
pub const INFO: Color32 = Color32::from_rgb(0x7A, 0xA2, 0xF7);

/// Radii.
pub const RADIUS_BADGE: u8 = 3;
pub const RADIUS_CONTROL: u8 = 5;
pub const RADIUS_CARD: u8 = 8;
pub const RADIUS_DIALOG: u8 = 12;

/// Fixed heights the design names.
pub const TITLE_BAR: f32 = 40.0;
pub const ACTION_BAR: f32 = 56.0;
pub const CONTROL: f32 = 34.0;

/// Font families, named the way the design speaks about them.
pub fn heading() -> FontFamily {
    FontFamily::Name("archivo".into())
}

pub fn body() -> FontFamily {
    FontFamily::Name("grotesk".into())
}

pub fn mono() -> FontFamily {
    FontFamily::Monospace
}

/// Installs the embedded faces and the dark palette.
pub fn install(context: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let faces = [
        ("archivo", Face::ArchivoExtraBold),
        ("grotesk", Face::SpaceGrotesk),
        ("plex", Face::MonoRegular),
    ];

    for (name, face) in faces {
        fonts
            .font_data
            .insert(name.to_owned(), FontData::from_static(face.bytes()).into());
    }

    fonts.families.insert(
        FontFamily::Name("archivo".into()),
        vec!["archivo".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name("grotesk".into()),
        vec!["grotesk".to_owned()],
    );
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "plex".to_owned());
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "grotesk".to_owned());

    context.set_fonts(fonts);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PAGE;
    visuals.window_fill = PAGE;
    visuals.extreme_bg_color = VOID;
    visuals.faint_bg_color = RAISED;
    visuals.override_text_color = Some(BODY);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.bg_fill = RAISED;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
    visuals.widgets.inactive.bg_fill = CHIP;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, BODY);
    visuals.widgets.hovered.bg_fill = CHIP;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, STRONG);
    visuals.widgets.active.bg_fill = ACCENT_PRESS;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, STRONG);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_CONTROL);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_CONTROL);
    visuals.widgets.active.corner_radius = CornerRadius::same(RADIUS_CONTROL);
    context.set_visuals(visuals);

    context.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
    });
}
