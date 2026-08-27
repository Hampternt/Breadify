//! Step 3: the few things about the printed page that are the user's to
//! decide, and a sample block that shows the decision immediately.

use eframe::egui::{self, CornerRadius, RichText, Stroke, Vec2};

use super::{Breadify, preview, theme};
use crate::crates::{self, STANDARD_SIZE};
use crate::layout::settings::{ALWAYS_PRINTED, NEVER_PRINTED};
use crate::layout::{Cursor, MarkerTreatment, stop};
use crate::order::Order;

/// Fields on the left, the choices in the middle, a real block on the right.
pub fn show(app: &mut Breadify, ui: &mut egui::Ui) {
    if app.loaded.is_none() {
        ui.label("No file open.");
        return;
    }

    let full = ui.available_size();
    let rail = 300.0;
    let sample = 452.0;

    ui.horizontal_top(|ui| {
        ui.allocate_ui(Vec2::new(rail, full.y), |ui| fields(app, ui));
        ui.add_space(16.0);
        ui.allocate_ui(Vec2::new(full.x - rail - sample - 48.0, full.y), |ui| {
            choices(app, ui)
        });
        ui.add_space(16.0);
        ui.allocate_ui(Vec2::new(sample, full.y), |ui| sample_block(app, ui));
    });
}

/// What prints, and the one line of it the user owns.
fn fields(app: &mut Breadify, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        heading(ui, "ALWAYS PRINTED");
        for field in ALWAYS_PRINTED {
            row(ui, field, "locked", theme::FAINT);
        }

        ui.add_space(12.0);
        heading(ui, "YOURS");
        let mut show = app.settings.show_order_id;
        if ui
            .checkbox(&mut show, RichText::new("Order ID").color(theme::BODY))
            .changed()
        {
            app.settings.show_order_id = show;
            app.resettle();
        }

        ui.add_space(12.0);
        heading(ui, "NEVER PRINTED");
        row(ui, NEVER_PRINTED.0, NEVER_PRINTED.1, theme::BORDER_STRONG);
    });
}

/// The marker treatment and the crate arithmetic.
fn choices(app: &mut Breadify, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_salt("choices")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                heading(ui, "NO-SUBSTITUTES MARKER");
                for treatment in MarkerTreatment::ALL {
                    let chosen = app.settings.marker == treatment;
                    if card(ui, treatment.label(), describe(treatment), chosen) {
                        app.settings.marker = treatment;
                        app.resettle();
                    }
                }

                ui.add_space(14.0);
                heading(ui, "CRATES");
                let mut large = app.settings.crates.large_capacity;
                let mut small = app.settings.crates.small_capacity;
                ui.horizontal(|ui| {
                    ui.label(RichText::new("large holds").color(theme::MUTED).size(12.0));
                    if ui
                        .add(egui::DragValue::new(&mut large).range(1..=99))
                        .changed()
                    {
                        app.settings.crates.large_capacity = large;
                        app.resettle();
                    }
                    ui.label(RichText::new("small holds").color(theme::MUTED).size(12.0));
                    if ui
                        .add(egui::DragValue::new(&mut small).range(1..=98))
                        .changed()
                    {
                        app.settings.crates.small_capacity = small;
                        app.resettle();
                    }
                });

                ui.add_space(14.0);
                heading(ui, "HOW MUCH ROOM EACH BREAD TAKES");
                ui.label(
                    RichText::new("A whole slot is 1. Click a bread to say it takes less.")
                        .family(theme::mono())
                        .size(11.0)
                        .color(theme::FAINT),
                );
                ui.add_space(6.0);
                sizes(app, ui);
            });
        });
}

/// Every product in the open file, with the room it takes.
///
/// A day's export runs to fifty-odd breads and nearly all of them are a whole
/// slot, so the list is one dense row each and says nothing about the ones
/// nobody has had to think about. The few that are not standard are lifted to
/// the top and carry their fraction; clicking any row opens the buttons.
fn sizes(app: &mut Breadify, ui: &mut egui::Ui) {
    let Some(loaded) = &app.loaded else {
        return;
    };

    let mut products: Vec<(u32, String)> = loaded
        .orders
        .iter()
        .flat_map(|order| order.lines.iter())
        .map(|line| (line.product.id, line.product.name.clone()))
        .collect();
    products.sort_by(|left, right| left.1.cmp(&right.1));
    products.dedup_by(|left, right| left.0 == right.0);

    let custom: Vec<(u32, String)> = products
        .iter()
        .filter(|(id, _)| app.settings.crates.is_custom(*id))
        .cloned()
        .collect();

    // Clicks are collected and applied after the list, so the whole list can
    // read the settings while it draws.
    let mut set: Option<(u32, u32)> = None;
    let mut open: Option<Option<u32>> = None;
    // A bread that has been given a size appears twice — lifted, and again in
    // its place in the list — so only the first of the two opens its buttons.
    let mut expanded = false;
    ui.spacing_mut().item_spacing.y = 1.0;

    if !custom.is_empty() {
        ui.add_space(4.0);
        heading(ui, "NOT A WHOLE SLOT");
        for (id, name) in &custom {
            let percent = app.settings.crates.size_of(*id);
            if size_row(ui, name, percent, app.sizing == Some(*id), true) {
                open = Some((app.sizing != Some(*id)).then_some(*id));
            }
            if app.sizing == Some(*id) && !expanded {
                expanded = true;
                set = size_buttons(ui, percent).map(|value| (*id, value)).or(set);
            }
        }
        ui.add_space(10.0);
        heading(ui, "EVERY BREAD");
    }

    for (id, name) in &products {
        let percent = app.settings.crates.size_of(*id);
        if size_row(ui, name, percent, app.sizing == Some(*id), false) {
            open = Some((app.sizing != Some(*id)).then_some(*id));
        }
        if app.sizing == Some(*id) && !expanded {
            expanded = true;
            set = size_buttons(ui, percent).map(|value| (*id, value)).or(set);
        }
    }

    if let Some(which) = open {
        app.sizing = which;
    }
    if let Some((id, percent)) = set {
        app.settings.crates.set_size(id, percent);
        app.resettle();
    }
}

/// One bread in the list. Standard ones are just a name; the rest carry the
/// fraction they were given. Returns true when it was clicked.
fn size_row(ui: &mut egui::Ui, name: &str, percent: u32, open: bool, lifted: bool) -> bool {
    let custom = percent != STANDARD_SIZE;
    let response = egui::Frame::NONE
        .fill(if open {
            theme::CHIP
        } else {
            egui::Color32::TRANSPARENT
        })
        .corner_radius(CornerRadius::same(theme::RADIUS_BADGE))
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(name).family(theme::body()).size(11.5).color(
                    if custom || lifted {
                        theme::STRONG
                    } else {
                        theme::MUTED
                    },
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if custom {
                        ui.label(
                            RichText::new(crates::spoken(percent))
                                .family(theme::mono())
                                .size(11.5)
                                .color(theme::ACCENT),
                        );
                    }
                });
            });
        })
        .response;

    response
        .interact(egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

/// The fractions of a slot, and a percentage for anything they do not cover.
/// Returns the size that was chosen, if one was.
fn size_buttons(ui: &mut egui::Ui, percent: u32) -> Option<u32> {
    let mut chosen = None;

    ui.horizontal_wrapped(|ui| {
        ui.add_space(6.0);
        for (label, value) in crates::SIZE_PRESETS {
            let on = percent == value;
            let button = egui::Button::new(
                RichText::new(label)
                    .family(theme::mono())
                    .size(11.5)
                    .color(if on { theme::STRONG } else { theme::BODY }),
            )
            .fill(if on { theme::ACCENT_PRESS } else { theme::CARD })
            .stroke(Stroke::new(
                1.0,
                if on {
                    theme::ACCENT_BORDER
                } else {
                    theme::BORDER
                },
            ))
            .min_size(Vec2::new(42.0, 23.0));

            if ui.add(button).clicked() {
                chosen = Some(value);
            }
        }
    });

    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.label(
            RichText::new("or a share of a slot")
                .family(theme::mono())
                .size(10.5)
                .color(theme::FAINT),
        );
        let mut value = percent;
        if ui
            .add(
                egui::DragValue::new(&mut value)
                    .range(1..=400)
                    .suffix(" %")
                    .speed(1.0),
            )
            .changed()
        {
            chosen = Some(value);
        }
    });
    ui.add_space(6.0);

    chosen
}

/// One real stop, drawn from the same display list the sheet is drawn from.
fn sample_block(app: &Breadify, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        heading(ui, "A BLOCK, TO SCALE");
        let Some(stop) = sample_stop(app) else {
            return;
        };

        let column = Cursor::new(0.0);
        let (page, height) = stop::block(&stop, &app.settings, &column);

        let width = ui.available_width().min(452.0);
        let scale = width / crate::geometry::PAGE_WIDTH as f32;
        let box_height = (height as f32 + 4.0) * scale;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, box_height), egui::Sense::hover());

        let painter = ui.painter_at(rect);
        preview::paper(&painter, rect.min, scale, height + 4.0);
        preview::draw(
            &painter,
            &page,
            rect.min + Vec2::new(0.0, 2.0 * scale),
            scale,
        );

        ui.add_space(10.0);
        let count = crates::count(&stop, &app.settings.crates);
        ui.label(
            RichText::new(format!(
                "{} units · {} slots · {} large + {} small",
                stop.units(),
                crates::slots(&stop, &app.settings.crates),
                count.large,
                count.small
            ))
            .family(theme::mono())
            .size(11.5)
            .color(theme::MUTED),
        );
    });
}

/// A stop worth looking at: one that exercises both of the choices above it —
/// a department, so the second line of the heading is there, and a refusal, so
/// the marker treatment is visible. Falling back through each half on its own
/// to the first stop in the file.
fn sample_stop(app: &Breadify) -> Option<Order> {
    let loaded = app.loaded.as_ref()?;
    let worth_seeing = |order: &&Order| order.lines.len() > 1;
    let has_department = |order: &&Order| order.department.is_some();
    let refuses = |order: &&Order| !order.accept_alternatives;

    let orders = || loaded.orders.iter();
    orders()
        .find(|order| worth_seeing(order) && has_department(order) && refuses(order))
        .or_else(|| orders().find(|order| worth_seeing(order) && has_department(order)))
        .or_else(|| orders().find(|order| worth_seeing(order) && refuses(order)))
        .or_else(|| loaded.orders.first())
        .cloned()
}

fn describe(treatment: MarkerTreatment) -> &'static str {
    match treatment {
        MarkerTreatment::InvertedBadge => "White on black, and a bar down the block. Two channels.",
        MarkerTreatment::HeavyRule => "The bar alone.",
        MarkerTreatment::WordOnly => "The words alone, in the heading.",
    }
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .family(theme::mono())
            .size(10.5)
            .color(theme::FAINT),
    );
    ui.add_space(6.0);
}

fn row(ui: &mut egui::Ui, name: &str, note: &str, note_colour: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(name)
                .family(theme::body())
                .size(12.5)
                .color(theme::BODY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(note)
                    .family(theme::mono())
                    .size(10.5)
                    .color(note_colour),
            );
        });
    });
}

/// A choosable card. Returns true when it was clicked.
fn card(ui: &mut egui::Ui, title: &str, detail: &str, chosen: bool) -> bool {
    let (fill, border) = if chosen {
        (theme::ACCENT_TINT, theme::ACCENT_BORDER)
    } else {
        (theme::CARD, theme::BORDER)
    };

    let response = egui::Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(10)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(title)
                        .family(theme::body())
                        .size(13.0)
                        .color(if chosen { theme::STRONG } else { theme::BODY }),
                );
                ui.label(
                    RichText::new(detail)
                        .family(theme::mono())
                        .size(10.5)
                        .color(theme::MUTED),
                );
            });
        })
        .response;

    ui.add_space(6.0);
    response.interact(egui::Sense::click()).clicked()
}
