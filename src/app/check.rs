//! Step 2: what was read, and what the file's own contents say about it.

use eframe::egui::{self, CornerRadius, RichText, Stroke, Vec2};

use super::{Breadify, Loaded};
use super::{mascot, theme};
use crate::date::ExportKind;
use crate::validate::{Finding, Severity};

/// The sentence the action bar carries on this step.
pub fn summary(app: &Breadify) -> String {
    let Some(loaded) = &app.loaded else {
        return "Nothing read yet.".to_owned();
    };

    let blocking = app.blocking_count();
    if blocking > 0 {
        return format!("{blocking} problem(s) would make the pages wrong.");
    }

    match loaded.findings.len() {
        0 => "Nothing to look at — every check passed.".to_owned(),
        1 => "One thing to look at — nothing that stops a print.".to_owned(),
        count => format!("{count} things to look at — nothing that stops a print."),
    }
}

/// Five stat cards, then one card per finding — over a bread roll nobody
/// asked for.
pub fn show(app: &mut Breadify, ui: &mut egui::Ui) {
    if app.loaded.is_none() {
        ui.label("No file open.");
        return;
    }

    mascot::behind(app, ui, ui.max_rect(), mascot::Mascot::BreadGuy);

    kind_banner(app, ui);
    ui.add_space(14.0);

    {
        let Some(loaded) = &app.loaded else {
            return;
        };
        stats(ui, loaded);
    }
    ui.add_space(14.0);

    let Some(loaded) = &app.loaded else {
        return;
    };

    let counts = tally(&loaded.findings);
    ui.label(
        RichText::new(counts)
            .family(theme::mono())
            .size(11.5)
            .color(theme::FAINT),
    );
    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for finding in &loaded.findings {
            card(ui, finding);
            ui.add_space(8.0);
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Sheet shape, required cells, column types, order consistency, \
                 product consistency, one route per address — all passed.",
            )
            .family(theme::mono())
            .size(11.5)
            .color(theme::FAINT),
        );
    });
}

/// The mode banner: which list the file is being treated as, worn as a
/// full-width band between the step rail and the stat cards, in the list's
/// own colour — crust yellow for bread, iced blue for freezer — so it cannot
/// be missed (decision F10). The filename decides it; the two buttons on the
/// band flip it when the name is wrong or custom, re-running the checks
/// below, since what is familiar depends on which list this is.
fn kind_banner(app: &mut Breadify, ui: &mut egui::Ui) {
    let freezer = app.settings.kind == ExportKind::Freezer;
    let (band, title) = if freezer {
        (theme::FREEZER_MODE, "FREEZER — CHECK LIST")
    } else {
        (theme::BREAD_MODE, "BREAD — PICKING LIST")
    };

    egui::Frame::NONE
        .fill(band)
        .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(
                        RichText::new("THIS FILE IS TREATED AS")
                            .family(theme::mono())
                            .size(10.0)
                            .color(theme::VOID),
                    );
                    ui.label(
                        RichText::new(title)
                            .family(theme::heading())
                            .size(21.0)
                            .color(theme::VOID),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Right-to-left: the first pill lands at the far right.
                    if mode_pill(ui, "FREEZER", freezer, band) {
                        app.set_kind(ExportKind::Freezer);
                    }
                    if mode_pill(ui, "BREAD", !freezer, band) {
                        app.set_kind(ExportKind::Bread);
                    }
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("read from the filename — wrong? click the other one")
                            .family(theme::mono())
                            .size(10.5)
                            .color(theme::VOID),
                    );
                });
            });
        });
}

/// One of the two mode buttons on the banner: the current answer is punched
/// dark through the band with the band's colour as its type; the other waits
/// as an outline.
fn mode_pill(ui: &mut egui::Ui, label: &str, chosen: bool, band: egui::Color32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(100.0, 32.0), egui::Sense::click());

    let fill = if chosen {
        theme::VOID
    } else if response.hovered() {
        egui::Color32::from_black_alpha(64)
    } else {
        egui::Color32::TRANSPARENT
    };
    let text = if chosen { band } else { theme::VOID };

    ui.painter().rect(
        rect,
        CornerRadius::same(theme::RADIUS_CONTROL),
        fill,
        Stroke::new(1.2, theme::VOID),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::new(13.0, theme::heading()),
        text,
    );

    response.clicked()
}

/// What was read, in five numbers.
fn stats(ui: &mut egui::Ui, loaded: &Loaded) {
    let date = loaded
        .dates
        .map_or_else(|| "none".to_owned(), |dates| dates.to_string());

    let cards = [
        (loaded.rows.len().to_string(), "ROWS"),
        (loaded.orders.len().to_string(), "ORDERS"),
        (loaded.routes.len().to_string(), "ROUTES"),
        (loaded.products().to_string(), "PRODUCTS"),
        (date, "DATE FROM FILENAME"),
    ];

    let gaps = ui.spacing().item_spacing.x * 4.0;
    let width = (ui.available_width() - gaps) / 5.0;
    ui.horizontal(|ui| {
        for (value, label) in cards {
            egui::Frame::NONE
                .fill(theme::CARD)
                .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
                .stroke(Stroke::new(1.0, theme::BORDER))
                .inner_margin(14)
                .show(ui, |ui| {
                    // The frame inherits the row's horizontal layout, so the
                    // value and its label need a column of their own.
                    ui.vertical(|ui| {
                        ui.set_width(width - 30.0);
                        ui.label(
                            RichText::new(value)
                                .family(theme::mono())
                                .size(24.0)
                                .color(theme::STRONG),
                        );
                        ui.label(
                            RichText::new(label)
                                .family(theme::mono())
                                .size(10.5)
                                .color(theme::FAINT),
                        );
                    });
                });
        }
    });
}

/// `0 blocking · 0 warnings · 2 notes`, from the findings themselves.
fn tally(findings: &[Finding]) -> String {
    let count = |severity: Severity| {
        findings
            .iter()
            .filter(|finding| finding.severity == severity)
            .count()
    };

    format!(
        "{} blocking · {} warnings · {} notes",
        count(Severity::Blocking),
        count(Severity::Warning),
        count(Severity::Notice)
    )
}

/// One finding: a badge column, the headline, and the detail beneath it.
fn card(ui: &mut egui::Ui, finding: &Finding) {
    let (colour, word) = match finding.severity {
        Severity::Blocking => (theme::DANGER, "blocking"),
        Severity::Warning => (theme::WARNING, "warning"),
        Severity::Notice => (theme::INFO, "note"),
    };

    egui::Frame::NONE
        .fill(theme::CARD_VEIL)
        .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
        .stroke(Stroke::new(1.0, theme::BORDER))
        .inner_margin(14)
        .show(ui, |ui| {
            // Every card is the same width, whatever its text.
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let (dot, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(dot.center(), 4.0, colour);

                ui.allocate_ui(Vec2::new(96.0, 20.0), |ui| {
                    ui.label(
                        RichText::new(word)
                            .family(theme::mono())
                            .size(11.0)
                            .color(colour),
                    );
                });

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&finding.headline)
                            .family(theme::body())
                            .size(14.0)
                            .color(theme::STRONG),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(&finding.detail)
                                .family(theme::mono())
                                .size(12.0)
                                .color(theme::MUTED),
                        )
                        .wrap(),
                    );
                    if !finding.rows.is_empty() {
                        ui.label(
                            RichText::new(rows_of(finding))
                                .family(theme::mono())
                                .size(11.0)
                                .color(theme::FAINT),
                        );
                    }
                });
            });
        });
}

/// `rows 21, 84, 132 and 9 more`, so the user can go and look.
fn rows_of(finding: &Finding) -> String {
    let shown: Vec<String> = finding
        .rows
        .iter()
        .take(3)
        .map(ToString::to_string)
        .collect();

    match finding.rows.len().saturating_sub(shown.len()) {
        0 => format!("rows {}", shown.join(", ")),
        rest => format!("rows {} and {rest} more", shown.join(", ")),
    }
}
