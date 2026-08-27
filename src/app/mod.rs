//! The app window: a four-step wizard around the printing the rest of the
//! crate already does.

pub mod check;
pub mod chrome;
pub mod configure;
pub mod mascot;
pub mod open;
pub mod preview;
pub mod print;
pub mod theme;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use eframe::egui;

use crate::date::DeliveryDates;
use crate::layout::Settings;
use crate::order::Order;
use crate::route::Route;
use crate::sheet::SheetRow;
use crate::validate::Finding;

/// Which of the four steps the user is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    Open,
    Check,
    Configure,
    Print,
}

impl Step {
    pub const ALL: [Step; 4] = [Step::Open, Step::Check, Step::Configure, Step::Print];

    /// `01`..`04`, as the rail sets them.
    pub fn number(self) -> &'static str {
        match self {
            Self::Open => "01",
            Self::Check => "02",
            Self::Configure => "03",
            Self::Print => "04",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Check => "Check",
            Self::Configure => "Configure",
            Self::Print => "Print",
        }
    }

    /// The step before this one, if there is one.
    pub fn previous(self) -> Option<Self> {
        match self {
            Self::Open => None,
            Self::Check => Some(Self::Open),
            Self::Configure => Some(Self::Check),
            Self::Print => Some(Self::Configure),
        }
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Self::Open => Some(Self::Check),
            Self::Check => Some(Self::Configure),
            Self::Configure => Some(Self::Print),
            Self::Print => None,
        }
    }
}

/// An export that has been read: everything the later steps work from.
pub struct Loaded {
    pub path: PathBuf,
    pub rows: Vec<SheetRow>,
    pub orders: Vec<Order>,
    pub routes: Vec<Route>,
    pub findings: Vec<Finding>,
    pub dates: Option<DeliveryDates>,
}

impl Loaded {
    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn products(&self) -> usize {
        let mut ids: Vec<u32> = self.rows.iter().map(|row| row.product_id).collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    }
}

/// What reading a file produced.
type LoadResult = Result<Box<Loaded>, String>;

/// The window's whole state.
pub struct Breadify {
    pub step: Step,
    pub loaded: Option<Loaded>,
    pub error: Option<String>,
    pub recent: Vec<PathBuf>,
    pub settings: Settings,
    /// Which routes are going to print.
    pub selected: BTreeSet<String>,
    /// The last file written, so the step can say where it went.
    pub wrote: Option<PathBuf>,
    /// The bread whose size buttons are open on the Configure step. The list
    /// is one row per product, so only one opens at a time.
    pub sizing: Option<u32>,
    /// The mascot, once a frame has asked for him.
    pub mascot: Option<egui::TextureHandle>,
    /// Where the crate rules were last written, so the step can say so.
    pub crates_kept: Option<PathBuf>,
    /// Set when the crate rules change; written out once the user lets go.
    crates_dirty: bool,
    /// The sheets the current selection comes to, worked out when it changes.
    day: Vec<crate::layout::Sheet>,
    /// Set while a file is being read on another thread.
    loading: Option<Receiver<LoadResult>>,
    /// Renders one frame and asks for a screenshot, for looking at the window
    /// without a person at the keyboard.
    screenshot: Option<PathBuf>,
    start_on: Option<Step>,
    stale: bool,
    frames: u32,
}

impl Breadify {
    pub fn new(
        context: &egui::Context,
        screenshot: Option<PathBuf>,
        open: Option<PathBuf>,
    ) -> Self {
        theme::install(context);
        // The crate rules are the warehouse's, not today's: they come back from
        // the last time somebody set them (crate::store).
        let settings = Settings {
            crates: crate::store::load().unwrap_or_default(),
            ..Settings::default()
        };
        let mut app = Self {
            step: Step::Open,
            loaded: None,
            error: None,
            recent: Vec::new(),
            settings,
            selected: BTreeSet::new(),
            wrote: None,
            sizing: None,
            mascot: None,
            crates_kept: None,
            crates_dirty: false,
            day: Vec::new(),
            loading: None,
            screenshot,
            start_on: None,
            stale: false,
            frames: 0,
        };
        if let Some(path) = open {
            app.load(path);
        }
        app
    }

    /// Opens on a given step once a file has loaded — for looking at a step
    /// without clicking to it.
    pub fn start_on(&mut self, index: usize) {
        self.start_on = Step::ALL.get(index).copied();
    }

    /// Whether a file is being read right now.
    pub fn is_loading(&self) -> bool {
        self.loading.is_some()
    }

    /// Starts reading `path` on another thread, so the window keeps painting.
    pub fn load(&mut self, path: PathBuf) {
        self.error = None;
        let (sender, receiver) = channel();
        self.loading = Some(receiver);

        std::thread::spawn(move || {
            let _ = sender.send(read(path));
        });
    }

    /// Picks up a finished read, if one has arrived.
    fn collect(&mut self) {
        let Some(receiver) = &self.loading else {
            return;
        };
        match receiver.try_recv() {
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.loading = None;
                self.error = Some("reading the file stopped unexpectedly".to_owned());
            }
            Ok(Err(message)) => {
                self.loading = None;
                self.error = Some(message);
            }
            Ok(Ok(loaded)) => {
                self.loading = None;
                self.remember(&loaded.path);
                self.selected = print::everything(&loaded.routes);
                self.loaded = Some(*loaded);
                self.step = self.start_on.unwrap_or(Step::Check);
                self.stale = true;
            }
        }
    }

    fn remember(&mut self, path: &PathBuf) {
        self.recent.retain(|seen| seen != path);
        self.recent.insert(0, path.clone());
        self.recent.truncate(6);
    }

    /// What the action bar's primary button says on this step.
    pub fn primary_label(&self) -> String {
        match self.step {
            Step::Open => "Read the file".to_owned(),
            Step::Check => match self.blocking_count() {
                0 => "Continue".to_owned(),
                _ => "Continue anyway".to_owned(),
            },
            Step::Configure => "Preview sheets".to_owned(),
            Step::Print => match self.selected_sheets() {
                0 => "Print".to_owned(),
                sheets => format!("Print {sheets} sheets"),
            },
        }
    }

    /// The sentence beside it.
    pub fn hint(&self) -> String {
        match self.step {
            Step::Open => {
                "One sheet named Data. The delivery date comes from the filename.".to_owned()
            }
            Step::Check => check::summary(self),
            Step::Configure => "Six fields always print; the order ID is yours.".to_owned(),
            Step::Print => {
                if let Some(path) = &self.wrote {
                    return format!(
                        "Wrote {}. Print at actual size — 100 %, no scaling.",
                        path.display()
                    );
                }
                "Print at actual size — 100 %, no scaling. One route per sheet set.".to_owned()
            }
        }
    }

    /// Marks what a changed setting invalidates.
    ///
    /// Paginating the day takes long enough to be felt at sixty frames a
    /// second, and a setting dragged through a range changes on every frame,
    /// so the work waits until the hand comes off the control.
    pub fn resettle(&mut self) {
        self.stale = true;
    }

    /// The same, for a change to the crate rules — which also outlive the
    /// window and so are written to disk once the user stops dragging.
    pub fn remember_crates(&mut self) {
        self.stale = true;
        self.crates_dirty = true;
    }

    /// Redoes the invalidated work, once the user has stopped dragging.
    fn settle(&mut self, context: &egui::Context) {
        if context.input(|input| input.pointer.any_down()) {
            return;
        }

        if self.crates_dirty {
            self.crates_dirty = false;
            self.keep_crates();
        }
        if !self.stale {
            return;
        }
        self.stale = false;
        self.day = print::selected_day(self);
    }

    /// Writes the crate rules out. A settings file that will not save is worth
    /// saying so on the step that owns it, and worth nothing more than that —
    /// it never stops a print.
    fn keep_crates(&mut self) {
        let names = self.loaded.as_ref().map_or_else(BTreeMap::new, |loaded| {
            loaded
                .orders
                .iter()
                .flat_map(|order| order.lines.iter())
                .map(|line| (line.product.id, line.product.name.clone()))
                .collect()
        });

        match crate::store::save(&self.settings.crates, &names) {
            Ok(path) => {
                // The action bar shows one message. A save that worked
                // supersedes whatever it was saying, the way a print that
                // worked does.
                self.error = None;
                self.crates_kept = Some(path);
            }
            Err(message) => self.error = Some(message),
        }
    }

    /// The sheets the current selection comes to. Worked out when something
    /// changes, and lent out rather than copied — a day is a few thousand
    /// positioned primitives, and this is read on every frame.
    ///
    /// There is one sheet count in the app and this is it: the rail, the route
    /// table and the print button all read it, so they cannot disagree about
    /// how much paper a print will take.
    pub fn day(&self) -> &[crate::layout::Sheet] {
        &self.day
    }

    /// How many sheets the selection comes to.
    pub fn selected_sheets(&self) -> usize {
        self.day.len()
    }

    /// The same count for the rail, or `None` while a change is settling.
    pub fn sheets_for_all(&self) -> Option<usize> {
        if self.stale {
            return None;
        }
        Some(self.day.len())
    }

    /// How many sheets one route needs on its own, or `None` while a changed
    /// setting has yet to be worked through.
    pub fn sheets_for(&self, route: &Route) -> Option<usize> {
        if self.stale {
            return None;
        }
        Some(
            self.day
                .iter()
                .filter(|sheet| sheet.route == route.nickname)
                .count(),
        )
    }

    pub fn blocking_count(&self) -> usize {
        self.loaded.as_ref().map_or(0, |loaded| {
            loaded
                .findings
                .iter()
                .filter(|finding| finding.severity == crate::validate::Severity::Blocking)
                .count()
        })
    }
}

/// Reads an export and derives everything the steps need from it.
fn read(path: PathBuf) -> LoadResult {
    let rows = crate::sheet::read(&path).map_err(|error| error.to_string())?;
    let findings = crate::validate::run(&rows);
    let orders = crate::order::fold(&rows);
    let routes = crate::route::group(orders.clone());
    let dates = crate::date::from_filename(&path).ok();
    Ok(Box::new(Loaded {
        path,
        rows,
        orders,
        routes,
        findings,
        dates,
    }))
}

impl eframe::App for Breadify {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        self.collect();
        self.settle(&context);
        if self.is_loading() {
            context.request_repaint();
        }

        chrome::title_bar(self, ui);
        chrome::step_rail(self, ui);
        chrome::action_bar(self, ui);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::PAGE).inner_margin(24))
            .show(ui, |ui| match self.step {
                Step::Open => open::show(self, ui),
                Step::Check => check::show(self, ui),
                Step::Configure => configure::show(self, ui),
                Step::Print => print::show(self, ui),
            });

        self.take_screenshot(&context);
    }
}

impl Breadify {
    /// Asks for a screenshot once the window has settled, writes it, and
    /// closes.
    fn take_screenshot(&mut self, context: &egui::Context) {
        let Some(target) = self.screenshot.clone() else {
            return;
        };

        self.frames += 1;
        if self.frames > 240 {
            eprintln!("breadify: the window never produced a screenshot");
            context.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if self.frames == 3 {
            context.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        if self.frames < 3 {
            context.request_repaint();
            return;
        }

        let captured = context.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });

        let Some(image) = captured else {
            context.request_repaint();
            return;
        };

        if let Err(error) = write_ppm(&target, &image) {
            eprintln!("breadify: could not write {}: {error}", target.display());
        }
        context.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

/// Writes a screenshot as a binary PPM — no image encoder, no dependency, and
/// every tool on the machine can read it.
fn write_ppm(path: &std::path::Path, image: &egui::ColorImage) -> std::io::Result<()> {
    use std::io::Write as _;

    let [width, height] = image.size;
    let mut out = Vec::with_capacity(width * height * 3 + 32);
    write!(out, "P6\n{width} {height}\n255\n")?;
    let (pixels, _) = image.as_raw().as_chunks::<4>();
    for pixel in pixels {
        out.extend_from_slice(&pixel[..3]);
    }
    std::fs::write(path, out)
}
