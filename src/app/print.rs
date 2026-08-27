//! Step 4: what to print, what it will look like, and getting it onto paper.

use std::collections::BTreeSet;
use std::path::PathBuf;

use eframe::egui::{self, CornerRadius, RichText, Stroke, Vec2};

use super::{Breadify, preview, theme};
use crate::geometry::{PAGE_HEIGHT, PAGE_WIDTH};
use crate::layout::{self, Sheet};
use crate::route::Route;

/// The routes on the left, their sheets as thumbnails on the right.
pub fn show(app: &mut Breadify, ui: &mut egui::Ui) {
    if app.loaded.is_none() {
        ui.label("No file open.");
        return;
    }

    let full = ui.available_size();
    let table = 360.0;

    ui.horizontal_top(|ui| {
        ui.allocate_ui(Vec2::new(table, full.y), |ui| routes(app, ui));
        ui.add_space(20.0);
        ui.allocate_ui(Vec2::new(full.x - table - 20.0, full.y), |ui| {
            thumbnails(app, ui);
        });
    });
}

/// Every route, what it costs in paper, and whether it is going to print.
fn routes(app: &mut Breadify, ui: &mut egui::Ui) {
    let Some(loaded) = &app.loaded else {
        return;
    };
    let rows: Vec<(String, usize, usize, Option<usize>)> = loaded
        .routes
        .iter()
        .map(|route| {
            (
                route.nickname.clone(),
                route.stops.len(),
                route.line_count(),
                app.sheets_for(route),
            )
        })
        .collect();

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if ui.button(RichText::new("All").size(12.0)).clicked() {
                app.selected = rows.iter().map(|(name, ..)| name.clone()).collect();
            }
            if ui.button(RichText::new("None").size(12.0)).clicked() {
                app.selected.clear();
            }
            ui.label(
                RichText::new(format!(
                    "{} of {} routes · {} sheets",
                    app.selected.len(),
                    rows.len(),
                    app.selected_sheets()
                ))
                .family(theme::mono())
                .size(11.0)
                .color(theme::MUTED),
            );
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .id_salt("routes")
            .show(ui, |ui| {
                for (nickname, stops, lines, sheets) in rows {
                    let mut chosen = app.selected.contains(&nickname);
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut chosen, "").changed() {
                            if chosen {
                                app.selected.insert(nickname.clone());
                            } else {
                                app.selected.remove(&nickname);
                            }
                        }
                        ui.label(
                            RichText::new(format!("Route {nickname}"))
                                .family(theme::body())
                                .size(13.0)
                                .color(theme::STRONG),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} · {lines} lines · {stops} stops",
                                    match sheets {
                                        None => "…".to_owned(),
                                        Some(1) => "1 sheet".to_owned(),
                                        Some(count) => format!("{count} sheets"),
                                    }
                                ))
                                .family(theme::mono())
                                .size(10.5)
                                .color(theme::FAINT),
                            );
                        });
                    });
                    ui.add_space(2.0);
                }
            });
    });
}

/// The sheets themselves, small enough to see the shape of a page.
fn thumbnails(app: &Breadify, ui: &mut egui::Ui) {
    let sheets = app.day();
    ui.vertical(|ui| {
        ui.label(
            RichText::new(if sheets.is_empty() {
                "Nothing selected.".to_owned()
            } else {
                format!("{} sheets, in printing order", sheets.len())
            })
            .family(theme::mono())
            .size(11.0)
            .color(theme::FAINT),
        );
        ui.add_space(8.0);

        let width = 132.0;
        let scale = width / PAGE_WIDTH as f32;
        let height = PAGE_HEIGHT as f32 * scale;
        let per_row = ((ui.available_width() + 12.0) / (width + 12.0))
            .floor()
            .max(1.0) as usize;

        egui::ScrollArea::vertical()
            .id_salt("thumbnails")
            .show(ui, |ui| {
                for chunk in sheets.chunks(per_row) {
                    ui.horizontal(|ui| {
                        for sheet in chunk {
                            thumbnail(ui, sheet, Vec2::new(width, height), scale);
                        }
                    });
                    ui.add_space(12.0);
                }
            });
    });
}

fn thumbnail(ui: &mut egui::Ui, sheet: &Sheet, size: Vec2, scale: f32) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    preview::paper(&painter, rect.min, scale, PAGE_HEIGHT);
    preview::draw(&painter, &sheet.content, rect.min, scale);
    painter.rect_stroke(
        rect,
        CornerRadius::same(2),
        Stroke::new(1.0, theme::BORDER),
        egui::StrokeKind::Outside,
    );
    painter.text(
        rect.center_bottom() + Vec2::new(0.0, 9.0),
        egui::Align2::CENTER_CENTER,
        format!("{} · {}/{}", sheet.route, sheet.number, sheet.of),
        egui::FontId::new(10.0, theme::mono()),
        theme::FAINT,
    );
    ui.add_space(2.0);
}

/// Writes the selected routes to a PDF the user names.
pub fn export(app: &mut Breadify) {
    let Some(target) = save_dialog(app) else {
        return;
    };
    write_and(app, &target, |_| Ok(()));
}

/// Writes the sheets and hands them to whatever the system opens PDFs with,
/// so the user gets their own printer picker, preview and scaling control
/// (decision D18).
pub fn hand_to_system(app: &mut Breadify) {
    let target = std::env::temp_dir().join(format!(
        "breadify-{}.pdf",
        app.loaded
            .as_ref()
            .and_then(|loaded| loaded.dates)
            .map_or_else(|| "sheets".to_owned(), |dates| dates.to_string())
    ));

    write_and(app, &target, |path| {
        opener::open(path).map_err(|error| format!("could not open the PDF: {error}"))
    });
}

fn write_and(
    app: &mut Breadify,
    target: &PathBuf,
    then: impl FnOnce(&PathBuf) -> Result<(), String>,
) {
    if app.day().is_empty() {
        app.error = Some("nothing selected to print".to_owned());
        return;
    }

    let pages: Vec<_> = app
        .day()
        .iter()
        .map(|sheet| sheet.content.clone())
        .collect();
    let outcome = crate::pdf::write(target, &pages, "Breadify pick lists")
        .map_err(|error| error.to_string())
        .and_then(|()| then(target));

    match outcome {
        Ok(()) => {
            app.error = None;
            app.wrote = Some(target.clone());
        }
        Err(message) => app.error = Some(message),
    }
}

fn save_dialog(app: &Breadify) -> Option<PathBuf> {
    // Named for the list as well as the day: both lists of one day used to
    // offer the same `2026-03-04.pdf`, and saving the second over the first is
    // a thing a person does once.
    let list = app.settings.list.to_string().to_lowercase();
    let name = app
        .loaded
        .as_ref()
        .and_then(|loaded| loaded.dates)
        .map_or_else(
            || format!("{list}-pick-lists.pdf"),
            |dates| format!("{list}-{dates}.pdf"),
        );

    rfd::FileDialog::new()
        .set_file_name(name)
        .add_filter("PDF", &["pdf"])
        .set_title("Save the pick lists")
        .save_file()
}

/// Which routes are selected by default: all of them.
pub fn everything(routes: &[Route]) -> BTreeSet<String> {
    routes.iter().map(|route| route.nickname.clone()).collect()
}

/// The sheets the selection comes to.
pub fn selected_day(app: &Breadify) -> Vec<Sheet> {
    let Some(loaded) = &app.loaded else {
        return Vec::new();
    };

    day_for(
        &loaded.routes,
        &app.selected,
        loaded.dates,
        &app.settings,
        &loaded.path.to_string_lossy(),
    )
}

/// The sheets a chosen set of routes comes to, in printing order.
///
/// Free of the window so it can be tested without one.
pub fn day_for(
    routes: &[Route],
    selected: &BTreeSet<String>,
    dates: Option<crate::date::DeliveryDates>,
    settings: &crate::layout::Settings,
    source: &str,
) -> Vec<Sheet> {
    let chosen: Vec<Route> = routes
        .iter()
        .filter(|route| selected.contains(&route.nickname))
        .cloned()
        .collect();

    layout::day(&chosen, dates, settings, source)
}
