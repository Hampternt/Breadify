//! What the filename says: which list this is, and the delivery date.
//!
//! No column of the export carries a date. The exporter names its files
//! `PSR-<KIND>-<from>-to-<to>.xlsx` — `PSR-BREAD-` for the bread list,
//! `PSR-FREEZER-` for the freezer list — and a browser that has downloaded
//! the same file twice appends ` (1)`. Anything that does not match is not an
//! error to throw at the user — it is a date to ask them for.

use std::fmt;
use std::path::Path;

/// Which of the exporter's lists a file is, told apart by its filename
/// prefix. The sheets inside are identical in shape; what differs is the
/// data — see `docs/freezer-format.md` — and eventually the printed page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    /// `PSR-BREAD-…` — the picking list the drivers pack crates from.
    Bread,
    /// `PSR-FREEZER-…` — the list a packed freezer box is checked against.
    Freezer,
}

impl ExportKind {
    /// The filename prefix that names this kind.
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Bread => "PSR-BREAD-",
            Self::Freezer => "PSR-FREEZER-",
        }
    }
}

/// Every kind the exporter produces, in the order they appeared.
pub const EXPORT_KINDS: [ExportKind; 2] = [ExportKind::Bread, ExportKind::Freezer];

/// A calendar day, to the precision the filename gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl fmt::Display for Date {
    /// ISO, which is how the printed page shows it.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

/// The span a file covers. Every export seen so far covers a single day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryDates {
    pub from: Date,
    pub to: Date,
}

impl DeliveryDates {
    /// Whether both ends are the same day, which is the ordinary case.
    pub fn is_single_day(&self) -> bool {
        self.from == self.to
    }
}

impl fmt::Display for DeliveryDates {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_single_day() {
            return write!(formatter, "{}", self.from);
        }
        write!(formatter, "{} to {}", self.from, self.to)
    }
}

/// A filename that does not carry dates. The caller should ask the user for
/// the delivery date rather than treat this as a failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "{filename:?} does not look like an export: \
     expected PSR-BREAD-<from>-to-<to>.xlsx or PSR-FREEZER-<from>-to-<to>.xlsx"
)]
pub struct UnnamedDates {
    pub filename: String,
}

const SEPARATOR: &str = "-to-";

/// Reads the delivery dates out of an export's filename.
///
/// # Errors
///
/// Returns [`UnnamedDates`] when the name does not carry two dates — which
/// means asking the user, not giving up.
pub fn from_filename(path: &Path) -> Result<DeliveryDates, UnnamedDates> {
    let filename = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    parse_stem(strip_download_suffix(stem(&filename)))
        .map(|(_, dates)| dates)
        .ok_or(UnnamedDates { filename })
}

/// Which list a file's name says it is, or `None` when the name says nothing —
/// a renamed file is still openable, it just cannot be told apart, and the
/// caller decides what to assume.
pub fn export_kind(path: &Path) -> Option<ExportKind> {
    let filename = path.file_name()?.to_string_lossy();
    EXPORT_KINDS
        .into_iter()
        .find(|kind| filename.starts_with(kind.prefix()))
}

/// `PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx` -> `PSR-BREAD-…-to-… (1)`.
fn stem(filename: &str) -> &str {
    filename.rsplit_once('.').map_or(filename, |(stem, _)| stem)
}

/// Drops a browser's ` (1)` duplicate marker.
fn strip_download_suffix(stem: &str) -> &str {
    let trimmed = stem.trim_end();
    let Some(open) = trimmed.rfind(" (") else {
        return trimmed;
    };
    let inside = &trimmed[open + 2..];
    let Some(number) = inside.strip_suffix(')') else {
        return trimmed;
    };
    if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) {
        return trimmed;
    }
    trimmed[..open].trim_end()
}

/// `PSR-BREAD-2026-03-04-to-2026-03-04` -> which list it is, and the two
/// dates.
fn parse_stem(stem: &str) -> Option<(ExportKind, DeliveryDates)> {
    let (kind, dates) = EXPORT_KINDS
        .into_iter()
        .find_map(|kind| Some((kind, stem.strip_prefix(kind.prefix())?)))?;
    let (from, to) = dates.split_once(SEPARATOR)?;
    Some((
        kind,
        DeliveryDates {
            from: parse_date(from)?,
            to: parse_date(to)?,
        },
    ))
}

/// `2026-03-04`, rejecting anything that is not a plausible calendar day.
fn parse_date(text: &str) -> Option<Date> {
    let mut parts = text.split('-');
    let year: u16 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(Date { year, month, day })
}
