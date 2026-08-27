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

    mascot::behind(app, ui);

    {
        let Some(loaded) = &app.loaded else {
            return;
        };
        stats(ui, loaded);
    }

    ui.add_space(14.0);
    kind_bar(app, ui);
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

/// Which list the file is being treated as: read from the filename, but the
/// user's to flip right here when the name is wrong or custom (decision
/// F10). Flipping re-runs the checks below it, since what is familiar
/// depends on which list this is.
fn kind_bar(app: &mut Breadify, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("TREATED AS")
                .family(theme::mono())
                .size(10.5)
                .color(theme::FAINT),
        );
        ui.add_space(4.0);

        for (kind, label) in [(ExportKind::Bread, "BREAD"), (ExportKind::Freezer, "FREEZER")] {
            if pill(ui, label, app.settings.kind == kind) {
                app.set_kind(kind);
            }
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("picking list or check list — read from the filename; flip it if the file was renamed")
                .family(theme::mono())
                .size(10.5)
                .color(theme::FAINT),
        );
    });
}

/// One of the two kind buttons: solid when it is the current answer, an
/// outline waiting to be clicked when it is not.
fn pill(ui: &mut egui::Ui, label: &str, chosen: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(92.0, 28.0), egui::Sense::click());

    let (fill, border, text) = if chosen {
        (theme::ACCENT, theme::ACCENT, theme::VOID)
    } else if response.hovered() {
        (theme::CARD, theme::BORDER_STRONG, theme::STRONG)
    } else {
        (theme::CARD, theme::BORDER, theme::MUTED)
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(theme::RADIUS_CONTROL),
        fill,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::new(12.0, theme::heading()),
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
