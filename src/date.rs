//! What the filename carries: which list, and which day.
//!
//! No column of the export carries a date, and none says which list it is
//! either. The exporter names its files `PSR-<list>-<from>-to-<to>.xlsx` —
//! `PSR-BREAD-…`, `PSR-FREEZER-…` — and a browser that has downloaded the same
//! file twice appends ` (1)`. Anything that does not match is not an error to
//! throw at the user — it is a date to ask them for.
//!
//! This module knows the shape of the name. What the list word *means* is
//! [`crate::list`]'s.

use std::fmt;
use std::path::Path;

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
#[error("{filename:?} does not look like an export: expected PSR-<list>-<from>-to-<to>.xlsx")]
pub struct UnnamedDates {
    pub filename: String,
}

const PREFIX: &str = "PSR-";
const SEPARATOR: &str = "-to-";
/// `2026-03-04` — the width every date in a filename has.
const DATE_WIDTH: usize = 10;

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

/// The list word out of an export's filename — `BREAD`, `FREEZER` — or
/// nothing when the name is not one the exporter wrote.
///
/// Reading the word rather than matching against a list of known ones is
/// deliberate: a kind this app has never met should still open, and be named
/// on screen as whatever it calls itself.
pub fn list_word(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_string_lossy().into_owned();
    let (word, _) = parse_stem(strip_download_suffix(stem(&filename)))?;
    Some(word)
}

/// `PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx` -> `PSR-BREAD-…-to-… (1)`.
///
/// A name with no dot keeps all of itself.
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

/// `PSR-BREAD-2026-03-04-to-2026-03-04` -> `BREAD` and the two dates.
///
/// The list word is taken as everything between the `PSR-` and the first
/// date, rather than up to the first hyphen, so a two-word kind would survive.
/// It is the date's fixed ten characters that make that possible.
fn parse_stem(stem: &str) -> Option<(String, DeliveryDates)> {
    let rest = stem.strip_prefix(PREFIX)?;
    let (head, to) = rest.rsplit_once(SEPARATOR)?;
    // `get` rather than `split_at`: a name whose bytes put a multi-byte
    // character across the cut would make `split_at` panic, and an oddly named
    // file is a file to shrug at, not to crash on.
    let cut = head.len().checked_sub(DATE_WIDTH)?;
    let word = head.get(..cut)?.strip_suffix('-')?;
    let from = head.get(cut..)?;
    if word.is_empty() {
        return None;
    }

    Some((
        word.to_owned(),
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
