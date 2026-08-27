//! Breadify — reads a daily bread-order export and prints A4 picking lists.
//!
//! The window and the printed page are still to come; for now the binary can
//! dump a route so the model can be read against `docs/print-spec.md` §10.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use breadify::layout;
use breadify::layout::Settings;
use breadify::list::Kind;
use breadify::route::Route;
use breadify::terminal;
use breadify::{date, dump, order, pdf, route, sheet, validate};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();

    let mut positional: Vec<&str> = Vec::new();
    let mut pdf: Option<&str> = None;
    let mut screenshot: Option<&str> = None;
    let mut open: Option<&str> = None;
    let mut step: Option<usize> = None;
    let mut rest = words.iter().copied();
    while let Some(word) = rest.next() {
        match word {
            "--pdf" => match rest.next() {
                Some(target) => pdf = Some(target),
                None => return fail("--pdf needs a file to write to"),
            },
            "--screenshot" => match rest.next() {
                Some(target) => screenshot = Some(target),
                None => return fail("--screenshot needs a file to write to"),
            },
            "--step" => match rest.next() {
                Some(index) => step = index.parse().ok(),
                None => return fail("--step needs a step number"),
            },
            "--open" => match rest.next() {
                Some(target) => open = Some(target),
                None => return fail("--open needs a file to read"),
            },
            "--help" | "-h" => return usage(),
            "--version" | "-V" => return version(),
            unknown if unknown.starts_with("--") => return usage(),
            value => positional.push(value),
        }
    }

    match positional.as_slice() {
        [] => window(screenshot.map(PathBuf::from), open.map(PathBuf::from), step),
        ["dump", nickname] => run(nickname, None, pdf),
        ["dump", nickname, path] => run(nickname, Some(Path::new(path)), pdf),
        ["help"] => usage(),
        ["version"] => version(),
        ["licences"] | ["licenses"] => licences(),
        ["print"] => print_day(None, pdf),
        ["print", path] => print_day(Some(Path::new(path)), pdf),
        _ => usage(),
    }
}

/// Opens the app window.
fn window(screenshot: Option<PathBuf>, open: Option<PathBuf>, step: Option<usize>) -> ExitCode {
    let (rgba, edge) = breadify::icon::window_icon();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 864.0])
            .with_min_inner_size([980.0, 640.0])
            .with_title("Breadify")
            .with_icon(eframe::egui::IconData {
                rgba,
                width: edge,
                height: edge,
            }),
        ..Default::default()
    };

    let outcome = eframe::run_native(
        "Breadify",
        options,
        Box::new(move |cc| {
            let mut app = breadify::app::Breadify::new(&cc.egui_ctx, screenshot, open);
            if let Some(index) = step {
                app.start_on(index);
            }
            Ok(Box::new(app))
        }),
    );

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&error.to_string()),
    }
}

/// The same rules the window uses: the warehouse's crate sizes as last saved,
/// the list the file's own name says it is, and the printed form's defaults
/// for everything else. A route dumped from the terminal has to come to the
/// same crates as the same route printed from the window, or one of them is
/// lying.
fn settings(path: &Path) -> Settings {
    Settings {
        crates: breadify::store::load().unwrap_or_default(),
        list: Kind::of(path),
        ..Settings::default()
    }
}

/// Draws every route in an export, one route per sheet set.
fn print_day(path: Option<&Path>, pdf: Option<&str>) -> ExitCode {
    let Some(target) = pdf else {
        return fail("print needs --pdf <file.pdf> to write to");
    };

    let path = match path
        .map(Path::to_path_buf)
        .ok_or(())
        .or_else(|()| export_here())
    {
        Ok(path) => path,
        Err(message) => return fail(&message),
    };

    let rows = match sheet::read(&path) {
        Ok(rows) => rows,
        Err(error) => return fail(&error.to_string()),
    };
    for finding in validate::run(&rows, &Kind::of(&path)) {
        warn(&format!("{:?}: {}\n", finding.severity, finding.headline));
    }

    let routes = route::group(order::fold(&rows));
    let dates = date::from_filename(&path).ok();
    let source = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();

    let sheets = layout::day(&routes, dates, &settings(&path), &source);
    let pages: Vec<_> = sheets.iter().map(|sheet| sheet.content.clone()).collect();

    match pdf::write(Path::new(target), &pages, "Breadify pick lists") {
        Ok(()) => {
            let mut out = format!(
                "{} routes · {} sheets · wrote {target}\n",
                routes.len(),
                sheets.len()
            );
            for sheet in &sheets {
                if sheet.of > 1 && sheet.number == 1 {
                    out.push_str(&format!(
                        "  route {} needs {} sheets\n",
                        sheet.route, sheet.of
                    ));
                }
            }
            say(&out)
        }
        Err(error) => fail(&error.to_string()),
    }
}

/// Everything the binary says on stdout goes through here. See
/// [`breadify::terminal`] for why it is not `println!`.
fn say(text: &str) -> ExitCode {
    match terminal::write(&mut std::io::stdout().lock(), text) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            warn(&format!("breadify: could not write output: {error}\n"));
            ExitCode::FAILURE
        }
    }
}

/// The same for stderr, where a failure has nowhere left to be reported to.
fn warn(text: &str) {
    let _ = terminal::write(&mut std::io::stderr().lock(), text);
}

/// Which build this is, so a report from the warehouse can be pinned to one.
fn version() -> ExitCode {
    say(&format!("breadify {}\n", env!("CARGO_PKG_VERSION")))
}

/// What the binary can do, for anyone who asks it wrongly.
fn usage() -> ExitCode {
    warn(
        "breadify — bread order exports into printed A4 picking lists

  breadify                                    open the window
  breadify dump <route> [export.xlsx]         print one route to the terminal
  breadify print [export.xlsx] --pdf <file>   draw every route
  breadify licences                           what is embedded, and under what terms
  breadify --version                          which build this is

Flags:
  --pdf <file.pdf>       also draw the route(s) as A4 sheets
  --open <export.xlsx>   open the window with a file already loaded
  --step <0-3>           open the window on a given step
  --screenshot <f.ppm>   render one frame, write it, and close

With no file given, looks for a single PSR-*.xlsx in this folder — the
bread list or the freezer one. Which list it is, and the delivery date,
are both read from the filename.
",
    );
    ExitCode::FAILURE
}

/// What ships inside the binary, and under what terms.
fn licences() -> ExitCode {
    let mut out = format!(
        "Breadify {} — © 2026 {}, MIT licensed. See LICENSE.\n\n\
         It embeds three typefaces, all under the SIL Open Font License 1.1.\n\
         The licence texts ship in assets/fonts/ and are reproduced in full there.\n\
         They stay under the OFL; the MIT licence above does not cover them.\n\n",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );

    for (family, source, licence) in [
        (
            "Archivo (ExtraBold, Black)",
            "github.com/Omnibus-Type/Archivo",
            "assets/fonts/Archivo-OFL.txt",
        ),
        (
            "Space Grotesk (Regular, Medium)",
            "github.com/floriankarsten/space-grotesk",
            "assets/fonts/SpaceGrotesk-OFL.txt",
        ),
        (
            "IBM Plex Mono (Regular, Medium, SemiBold, Bold)",
            "github.com/google/fonts",
            "assets/fonts/IBMPlexMono-OFL.txt",
        ),
    ] {
        out.push_str(&format!(
            "  {family}\n    from {source}\n    licence {licence}\n"
        ));
    }

    out.push_str(
        "\nThe Matvare Expressen wordmark is the customer's own and is not licensed here.\n",
    );
    say(&out)
}

fn run(nickname: &str, path: Option<&Path>, pdf: Option<&str>) -> ExitCode {
    let path = match path
        .map(Path::to_path_buf)
        .ok_or(())
        .or_else(|()| export_here())
    {
        Ok(path) => path,
        Err(message) => return fail(&message),
    };

    let rows = match sheet::read(&path) {
        Ok(rows) => rows,
        Err(error) => return fail(&error.to_string()),
    };

    for finding in validate::run(&rows, &Kind::of(&path)) {
        warn(&format!("{:?}: {}\n", finding.severity, finding.headline));
    }

    let routes = route::group(order::fold(&rows));
    let Some(wanted) = routes.iter().find(|route| route.nickname == nickname) else {
        return fail(&format!(
            "no route {nickname:?} in this export — it has {}",
            nicknames(&routes)
        ));
    };

    let dates = date::from_filename(&path).ok();
    let settings = settings(&path);
    let dumped = say(&dump::route(wanted, dates, &settings));
    if dumped != ExitCode::SUCCESS {
        return dumped;
    }

    let Some(target) = pdf else {
        return ExitCode::SUCCESS;
    };

    let source = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sheets = layout::paginate(wanted, dates, &settings, &source);
    let pages: Vec<_> = sheets.iter().map(|sheet| sheet.content.clone()).collect();

    match pdf::write(Path::new(target), &pages, &format!("Route {nickname}")) {
        Ok(()) => {
            warn(&format!("wrote {target} — {} sheet(s)\n", pages.len()));
            ExitCode::SUCCESS
        }
        Err(error) => fail(&error.to_string()),
    }
}

/// The single `PSR-*.xlsx` in the working directory, if there is exactly one —
/// the common case while working on a day's orders. With both a bread and a
/// freezer export to hand it asks which, which is the right answer: they are
/// separate lists and printing the wrong one wastes a ream.
fn export_here() -> Result<PathBuf, String> {
    let entries =
        std::fs::read_dir(".").map_err(|error| format!("cannot read this folder: {error}"))?;

    let mut exports: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_export(path))
        .collect();
    exports.sort();

    match exports.len() {
        0 => Err("no PSR-*.xlsx here — give the file as the second argument".to_owned()),
        1 => Ok(exports.remove(0)),
        _ => Err(format!(
            "several exports here — say which: {}",
            exports
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn is_export(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|name| name.to_string_lossy()) else {
        return false;
    };
    name.starts_with("PSR-") && name.ends_with(".xlsx")
}

fn nicknames(routes: &[Route]) -> String {
    routes
        .iter()
        .map(|route| route.nickname.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

fn fail(message: &str) -> ExitCode {
    warn(&format!("breadify: {message}\n"));
    ExitCode::FAILURE
}
