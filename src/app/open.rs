//! Step 1: getting today's export into the app.

use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};

use super::Breadify;
use super::mascot::{self, Mascot};
use super::theme;

/// The drop zone on the left, what has been opened before on the right.
pub fn show(app: &mut Breadify, ui: &mut egui::Ui) {
    accept_dropped_file(app, &ui.ctx().clone());

    let rail = 320.0;
    let available = ui.available_size();

    ui.horizontal_top(|ui| {
        ui.allocate_ui(Vec2::new(available.x - rail - 20.0, available.y), |ui| {
            drop_zone(app, ui);
        });
        ui.add_space(20.0);
        ui.allocate_ui(Vec2::new(rail, available.y), |ui| {
            recent(app, ui);
        });
    });
}

/// A file dragged onto the window is the fastest way in, so it is the one the
/// panel is built around.
fn accept_dropped_file(app: &mut Breadify, context: &egui::Context) {
    let dropped = context.input(|input| {
        input
            .raw
            .dropped_files
            .iter()
            .map(|file| file.path().to_path_buf())
            .next()
    });

    if let Some(path) = dropped {
        app.load(path);
    }
}

fn drop_zone(app: &mut Breadify, ui: &mut egui::Ui) {
    let size = ui.available_size();
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();

    painter.rect(
        rect,
        CornerRadius::same(theme::RADIUS_DIALOG),
        theme::RAISED,
        Stroke::new(1.0, theme::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );
    blueprint_grid(painter, rect);

    // Only while nothing has been opened — which is the one moment in the day
    // when there is, in fact, no bread. Coming back here from the rail with a
    // file already read is not that moment.
    if app.loaded.is_none() {
        mascot::behind(app, ui, rect, Mascot::NoBread);
    }
    let painter = ui.painter();

    let centre = rect.center();
    let waiting = app.is_loading();

    spreadsheet_glyph(painter, centre - egui::vec2(0.0, 62.0), waiting);
    painter.text(
        centre - egui::vec2(0.0, 14.0),
        egui::Align2::CENTER_CENTER,
        if waiting {
            "Reading the file"
        } else {
            "Drop today's export here"
        },
        egui::FontId::new(26.0, theme::heading()),
        theme::STRONG,
    );
    painter.text(
        centre + egui::vec2(0.0, 16.0),
        egui::Align2::CENTER_CENTER,
        "PSR-BREAD-<from>-to-<to>.xlsx — one sheet named Data, 14 headers plus one",
        egui::FontId::new(12.5, theme::mono()),
        theme::MUTED,
    );
    painter.text(
        centre + egui::vec2(0.0, 34.0),
        egui::Align2::CENTER_CENTER,
        "unlabelled column. The delivery date is read from the filename.",
        egui::FontId::new(12.5, theme::mono()),
        theme::MUTED,
    );

    let button =
        egui::Rect::from_center_size(centre + egui::vec2(0.0, 78.0), Vec2::new(150.0, 36.0));
    let response = ui.interact(button, ui.id().with("choose"), egui::Sense::click());
    let fill = if response.hovered() {
        theme::ACCENT_HOVER
    } else {
        theme::ACCENT
    };
    ui.painter()
        .rect_filled(button, CornerRadius::same(theme::RADIUS_CONTROL), fill);
    ui.painter().text(
        button.center(),
        egui::Align2::CENTER_CENTER,
        "Choose file",
        egui::FontId::new(14.0, theme::heading()),
        theme::VOID,
    );

    if response.clicked() {
        choose_file(app);
    }

    if let Some(error) = &app.error {
        ui.painter().text(
            centre + egui::vec2(0.0, 122.0),
            egui::Align2::CENTER_CENTER,
            error,
            egui::FontId::new(12.5, theme::mono()),
            theme::DANGER,
        );
    }
}

/// A sheet with a ruled grid — drawn rather than set, since none of the three
/// embedded faces carries a document glyph.
fn spreadsheet_glyph(painter: &egui::Painter, centre: egui::Pos2, waiting: bool) {
    let colour = if waiting { theme::ACCENT } else { theme::MUTED };
    let sheet = egui::Rect::from_center_size(centre, Vec2::new(26.0, 32.0));
    painter.rect_stroke(
        sheet,
        CornerRadius::same(theme::RADIUS_BADGE),
        Stroke::new(1.5, colour),
        egui::StrokeKind::Inside,
    );

    for row in 1..4 {
        let y = sheet.top() + sheet.height() * row as f32 / 4.0;
        painter.line_segment(
            [
                egui::pos2(sheet.left() + 4.0, y),
                egui::pos2(sheet.right() - 4.0, y),
            ],
            Stroke::new(1.0, colour),
        );
    }
    painter.line_segment(
        [
            egui::pos2(sheet.center().x, sheet.top() + 4.0),
            egui::pos2(sheet.center().x, sheet.bottom() - 4.0),
        ],
        Stroke::new(1.0, colour),
    );
}

/// The faint grid that tells the panel apart from the page behind it.
fn blueprint_grid(painter: &egui::Painter, rect: egui::Rect) {
    let step = 32.0;
    let ink = Color32::from_rgba_premultiplied(0x0B, 0x0B, 0x0B, 0x0B);

    let mut x = rect.left() + step;
    while x < rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            Stroke::new(1.0, ink),
        );
        x += step;
    }

    let mut y = rect.top() + step;
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(1.0, ink),
        );
        y += step;
    }
}

/// Opens the system file dialog.
fn choose_file(app: &mut Breadify) {
    let picked = rfd::FileDialog::new()
        .add_filter("Bread order export", &["xlsx"])
        .set_title("Choose today's export")
        .pick_file();

    if let Some(path) = picked {
        app.load(path);
    }
}

/// What has been opened before, newest first.
fn recent(app: &mut Breadify, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new("RECENT")
                .family(theme::mono())
                .size(11.0)
                .color(theme::FAINT),
        );
        ui.add_space(8.0);

        if app.recent.is_empty() {
            ui.label(
                RichText::new("Nothing opened yet.")
                    .family(theme::mono())
                    .size(12.0)
                    .color(theme::FAINT),
            );
        }

        let entries = app.recent.clone();
        for path in entries {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let card = egui::Frame::NONE
                .fill(theme::CARD)
                .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
                .stroke(Stroke::new(1.0, theme::BORDER))
                .inner_margin(12.0);

            let response = card
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(&name)
                            .family(theme::mono())
                            .size(12.5)
                            .color(theme::BODY),
                    );
                    ui.label(
                        RichText::new(
                            path.parent()
                                .map(|parent| parent.display().to_string())
                                .unwrap_or_default(),
                        )
                        .family(theme::mono())
                        .size(11.0)
                        .color(theme::FAINT),
                    );
                })
                .response;

            if response.interact(egui::Sense::click()).clicked() {
                app.load(path);
            }
            ui.add_space(8.0);
        }
    });
}
