//! Breadify — reads a daily bread-order export and prints A4 picking lists.
//!
//! The window and the printed page are still to come; for now the binary can
//! dump a route so the model can be read against `docs/print-spec.md` §10.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use breadify::layout;
use breadify::layout::Settings;
use breadify::route::Route;
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
            unknown if unknown.starts_with("--") => return usage(),
            value => positional.push(value),
        }
    }

    match positional.as_slice() {
        [] => window(screenshot.map(PathBuf::from), open.map(PathBuf::from), step),
        ["dump", nickname] => run(nickname, None, pdf),
        ["dump", nickname, path] => run(nickname, Some(Path::new(path)), pdf),
        ["help"] => usage(),
        ["licences"] | ["licenses"] => licences(),
        ["print"] => print_day(None, pdf),
        ["print", path] => print_day(Some(Path::new(path)), pdf),
        _ => usage(),
    }
}

/// Opens the app window.
fn window(screenshot: Option<PathBuf>, open: Option<PathBuf>, step: Option<usize>) -> ExitCode {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 864.0])
            .with_min_inner_size([980.0, 640.0])
            .with_title("Breadify"),
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
    for finding in validate::run(&rows) {
        eprintln!("{:?}: {}", finding.severity, finding.headline);
    }

    let routes = route::group(order::fold(&rows));
    let dates = date::from_filename(&path).ok();
    let source = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();

    let sheets = layout::day(&routes, dates, &Settings::default(), &source);
    let pages: Vec<_> = sheets.iter().map(|sheet| sheet.content.clone()).collect();

    match pdf::write(Path::new(target), &pages, "Breadify pick lists") {
        Ok(()) => {
            println!(
                "{} routes · {} sheets · wrote {target}",
                routes.len(),
                sheets.len()
            );
            for sheet in &sheets {
                if sheet.of > 1 && sheet.number == 1 {
                    println!("  route {} needs {} sheets", sheet.route, sheet.of);
                }
            }
            ExitCode::SUCCESS
        }
        Err(error) => fail(&error.to_string()),
    }
}

/// What the binary can do, for anyone who asks it wrongly.
fn usage() -> ExitCode {
    eprintln!("breadify — bread order exports into printed A4 picking lists");
    eprintln!();
    eprintln!("  breadify                                    open the window");
    eprintln!("  breadify dump <route> [export.xlsx]         print one route to the terminal");
    eprintln!("  breadify print [export.xlsx] --pdf <file>   draw every route");
    eprintln!(
        "  breadify licences                           what is embedded, and under what terms"
    );
    eprintln!();
    eprintln!("Flags:");
    eprintln!("  --pdf <file.pdf>       also draw the route(s) as A4 sheets");
    eprintln!("  --open <export.xlsx>   open the window with a file already loaded");
    eprintln!("  --step <0-3>           open the window on a given step");
    eprintln!("  --screenshot <f.ppm>   render one frame, write it, and close");
    eprintln!();
    eprintln!("With no file given, looks for a single PSR-BREAD-*.xlsx in this folder.");
    eprintln!("The delivery date is read from the filename.");
    ExitCode::FAILURE
}

/// What ships inside the binary, and under what terms.
fn licences() -> ExitCode {
    println!("Breadify embeds three typefaces, all under the SIL Open Font License 1.1.");
    println!("The licence texts ship in assets/fonts/ and are reproduced in full there.");
    println!();
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
        println!("  {family}");
        println!("    from {source}");
        println!("    licence {licence}");
    }
    println!();
    println!("The Matvare Expressen wordmark is the customer's own and is not licensed here.");
    ExitCode::SUCCESS
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

    for finding in validate::run(&rows) {
        eprintln!("{:?}: {}", finding.severity, finding.headline);
    }

    let routes = route::group(order::fold(&rows));
    let Some(wanted) = routes.iter().find(|route| route.nickname == nickname) else {
        return fail(&format!(
            "no route {nickname:?} in this export — it has {}",
            nicknames(&routes)
        ));
    };

    let dates = date::from_filename(&path).ok();
    let settings = Settings::default();
    print!("{}", dump::route(wanted, dates, &settings.crates));

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
            eprintln!("wrote {target} — {} sheet(s)", pages.len());
            ExitCode::SUCCESS
        }
        Err(error) => fail(&error.to_string()),
    }
}

/// The single `PSR-BREAD-*.xlsx` in the working directory, if there is exactly
/// one — the common case while working on a day's orders.
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
        0 => Err("no PSR-BREAD-*.xlsx here — give the file as the second argument".to_owned()),
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
    name.starts_with("PSR-BREAD-") && name.ends_with(".xlsx")
}

fn nicknames(routes: &[Route]) -> String {
    routes
        .iter()
        .map(|route| route.nickname.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

fn fail(message: &str) -> ExitCode {
    eprintln!("breadify: {message}");
    ExitCode::FAILURE
}
