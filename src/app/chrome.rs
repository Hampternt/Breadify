//! The window's fixed furniture: title bar, step rail and action bar.

use eframe::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Stroke, Vec2};

use super::theme::{self, ACTION_BAR, TITLE_BAR};
use super::{Breadify, Step};

/// The 40 px bar: wordmark, the loaded filename, and the window buttons.
pub fn title_bar(app: &mut Breadify, host: &mut egui::Ui) {
    egui::Panel::top("title-bar")
        .exact_size(TITLE_BAR)
        .frame(
            egui::Frame::NONE
                .fill(theme::RAISED)
                .inner_margin(egui::Margin::symmetric(14, 0)),
        )
        .show(host, |ui| {
            ui.horizontal_centered(|ui| {
                wordmark(ui);
                ui.add_space(12.0);

                let filename = app
                    .loaded
                    .as_ref()
                    .map(super::Loaded::filename)
                    .unwrap_or_else(|| "no file open".to_owned());
                ui.label(
                    RichText::new(filename)
                        .family(theme::mono())
                        .size(11.0)
                        .color(theme::FAINT),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new(
                            RichText::new("×").size(16.0).color(theme::MUTED),
                        ))
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
}

/// The Matvare Expressen wordmark on its dark panel, as on the printed sheet.
fn wordmark(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(96.0, 22.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_BADGE), theme::VOID);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "MATVARE EXPRESSEN",
        egui::FontId::new(9.0, theme::heading()),
        theme::STRONG,
    );
}

/// Four clickable tabs. The wizard is not one-way.
pub fn step_rail(app: &mut Breadify, host: &mut egui::Ui) {
    egui::Panel::top("step-rail")
        .exact_size(58.0)
        .frame(egui::Frame::NONE.fill(theme::RAISED))
        .show(host, |ui| {
            let width = ui.available_width() / 4.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for step in Step::ALL {
                    tab(app, ui, step, width);
                }
            });
        });
}

fn tab(app: &mut Breadify, ui: &mut egui::Ui, step: Step, width: f32) {
    let reachable = step == Step::Open || app.loaded.is_some();
    let active = app.step == step;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, 58.0),
        if reachable {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );

    if active {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, theme::CARD);
        let underline = egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - 2.0),
            rect.right_bottom(),
        );
        ui.painter()
            .rect_filled(underline, CornerRadius::ZERO, theme::ACCENT);
    }

    let (number_colour, label_colour) = match (active, reachable) {
        (true, _) => (theme::ACCENT_HOVER, theme::STRONG),
        (_, true) => (theme::FAINT, theme::BODY),
        _ => (theme::BORDER_STRONG, theme::FAINT),
    };

    let painter = ui.painter();
    painter.text(
        egui::pos2(rect.left() + 18.0, rect.center().y - 8.0),
        egui::Align2::LEFT_CENTER,
        step.number(),
        egui::FontId::new(11.0, theme::mono()),
        number_colour,
    );
    painter.text(
        egui::pos2(rect.left() + 18.0, rect.center().y + 9.0),
        egui::Align2::LEFT_CENTER,
        step.label(),
        egui::FontId::new(15.0, theme::heading()),
        label_colour,
    );
    painter.text(
        egui::pos2(rect.right() - 18.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        note(app, step),
        egui::FontId::new(11.0, theme::mono()),
        theme::FAINT,
    );

    if response.clicked() && reachable {
        app.step = step;
    }
}

/// The right-aligned note on a tab: what that step has to say right now.
fn note(app: &Breadify, step: Step) -> String {
    let Some(loaded) = &app.loaded else {
        return match step {
            Step::Open => "waiting".to_owned(),
            _ => String::new(),
        };
    };

    match step {
        Step::Open => loaded
            .dates
            .map_or_else(|| "no date".to_owned(), |dates| dates.to_string()),
        Step::Check => {
            let findings = loaded.findings.len();
            match findings {
                0 => "all clear".to_owned(),
                1 => "1 finding".to_owned(),
                _ => format!("{findings} findings"),
            }
        }
        Step::Configure => format!("{} routes", loaded.routes.len()),
        Step::Print => {
            let sheets = crate::layout::day(
                &loaded.routes,
                loaded.dates,
                &app.settings,
                &loaded.filename(),
            )
            .len();
            format!("{sheets} sheets")
        }
    }
}

/// Back and a hint on the left, the step's primary action on the right.
pub fn action_bar(app: &mut Breadify, host: &mut egui::Ui) {
    egui::Panel::bottom("action-bar")
        .exact_size(ACTION_BAR)
        .frame(
            egui::Frame::NONE
                .fill(theme::RAISED)
                .inner_margin(egui::Margin::symmetric(20, 0)),
        )
        .show(host, |ui| {
            ui.horizontal_centered(|ui| {
                let back = ui.add_enabled(
                    app.step.previous().is_some(),
                    egui::Button::new(RichText::new("Back").color(theme::BODY))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, theme::BORDER)),
                );
                if back.clicked()
                    && let Some(previous) = app.step.previous()
                {
                    app.step = previous;
                }

                ui.add_space(14.0);
                // Whatever went wrong is said here, on whichever step it went
                // wrong on. The drop zone shows it too, but a settings file
                // that will not save or a PDF that will not open happens three
                // steps away from there.
                let (message, colour) = match &app.error {
                    Some(error) => (error.clone(), theme::DANGER),
                    None => (app.hint(), theme::MUTED),
                };
                ui.label(
                    RichText::new(message)
                        .family(theme::mono())
                        .size(11.5)
                        .color(colour),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let ready = app.loaded.is_some();
                    let primary = ui.add_enabled(
                        ready && !(app.step == Step::Print && app.selected_sheets() == 0),
                        egui::Button::new(RichText::new(app.primary_label()).color(theme::VOID))
                            .fill(theme::ACCENT)
                            .min_size(Vec2::new(0.0, theme::CONTROL)),
                    );

                    if primary.clicked() {
                        if app.step == Step::Print {
                            crate::app::print::hand_to_system(app);
                        } else if let Some(next) = app.step.next() {
                            app.step = next;
                        }
                    }

                    if app.step == Step::Print {
                        let export = ui.add_enabled(
                            ready && app.selected_sheets() > 0,
                            egui::Button::new(RichText::new("Export PDF").color(theme::BODY))
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, theme::BORDER)),
                        );
                        if export.clicked() {
                            crate::app::print::export(app);
                        }
                    }
                });
            });
        });
}
