//! Step 2: what was read, and what the file's own contents say about it.

use eframe::egui::{self, CornerRadius, RichText, Stroke, Vec2};

use super::theme;
use super::{Breadify, Loaded};
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

/// Five stat cards, then one card per finding.
pub fn show(app: &mut Breadify, ui: &mut egui::Ui) {
    let Some(loaded) = &app.loaded else {
        ui.label("No file open.");
        return;
    };

    stats(ui, loaded);
    ui.add_space(20.0);

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
        .fill(theme::CARD)
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
