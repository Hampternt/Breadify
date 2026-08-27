//! Breadify — reads a daily bread-order export and prints A4 picking lists.
//!
//! The window and the printed page are still to come; for now the binary can
//! dump a route so the model can be read against `docs/print-spec.md` §10.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use breadify::crates::CrateRules;
use breadify::layout::{self, SheetContext};
use breadify::route::Route;
use breadify::{date, dump, order, pdf, route, sheet, validate};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();

    let mut positional: Vec<&str> = Vec::new();
    let mut pdf: Option<&str> = None;
    let mut rest = words.iter().copied();
    while let Some(word) = rest.next() {
        match word {
            "--pdf" => match rest.next() {
                Some(target) => pdf = Some(target),
                None => return fail("--pdf needs a file to write to"),
            },
            unknown if unknown.starts_with("--") => return usage(),
            value => positional.push(value),
        }
    }

    match positional.as_slice() {
        ["dump", nickname] => run(nickname, None, pdf),
        ["dump", nickname, path] => run(nickname, Some(Path::new(path)), pdf),
        ["print"] => print_day(None, pdf),
        ["print", path] => print_day(Some(Path::new(path)), pdf),
        _ => usage(),
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

    let sheets = layout::day(&routes, dates, &CrateRules::default(), &source);
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
    eprintln!("usage: breadify dump <route> [export.xlsx] [--pdf <file.pdf>]");
    eprintln!("       breadify print [export.xlsx] --pdf <file.pdf>");
    eprintln!();
    eprintln!("Prints one route's stops, crates and total. With no file given,");
    eprintln!("looks for a single PSR-BREAD-*.xlsx in the current directory.");
    eprintln!("With --pdf, also draws the route as an A4 sheet.");
    ExitCode::FAILURE
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
    let rules = CrateRules::default();
    print!("{}", dump::route(wanted, dates, &rules));

    let Some(target) = pdf else {
        return ExitCode::SUCCESS;
    };

    let source = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let context = SheetContext::single(wanted, dates, source);
    let sheet = layout::sheet(wanted, &context, &rules);

    match pdf::write(Path::new(target), &[sheet], &format!("Route {nickname}")) {
        Ok(()) => {
            eprintln!("wrote {target}");
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
