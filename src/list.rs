//! Which list an export is: bread, or freezer.
//!
//! Both come out of the same exporter in the same shape — one `Data` sheet,
//! the same fourteen headers plus the unlabelled region column — and nothing
//! inside the sheet says them apart. The only thing that does is the word in
//! the filename: `PSR-BREAD-…` against `PSR-FREEZER-…`.
//!
//! A file whose name carries no kind is read as bread, because that is what
//! this app was built for and what every export before the freezer one was.
//! The Check step says which list it decided on, so a renamed file is a thing
//! the user can see rather than a thing the app gets quietly wrong.

use std::fmt;
use std::path::Path;

use crate::date;

/// The list an export holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// The morning's bread, from the two bakeries.
    Bread,
    /// Frozen goods, from whoever supplies them.
    Freezer,
    /// A word this app has not met. Treated as freezer in everything it
    /// touches, because the bread-shaped parts of the app — the crate
    /// arithmetic above all — are the ones that would be confidently wrong.
    Other(String),
}

impl Kind {
    /// Which list the file at `path` holds, read from its name.
    pub fn of(path: &Path) -> Self {
        date::list_word(path).map_or(Self::Bread, |word| Self::from_word(&word))
    }

    /// The word out of a filename, whatever its case.
    pub fn from_word(word: &str) -> Self {
        if word.eq_ignore_ascii_case("bread") {
            return Self::Bread;
        }
        if word.eq_ignore_ascii_case("freezer") {
            return Self::Freezer;
        }
        Self::Other(word.to_owned())
    }

    /// Whether crates mean anything on this list.
    ///
    /// The arithmetic is bread-shaped: fifty units to a large crate, each
    /// product a fraction of a slot. `Lasagne 2,5 Kg` and `Hamburgerbrød 48stk
    /// Eske` are not slot-shaped, and a wrong crate count on a sheet the
    /// driver trusts is worse than none at all.
    pub fn has_crates(&self) -> bool {
        matches!(self, Self::Bread)
    }

    /// The word that goes in front of a count of them — `11 bread types`,
    /// `18 frozen types`. Empty for a kind this app has no word for, which
    /// leaves `18 types`.
    pub fn modifier(&self) -> &str {
        match self {
            Self::Bread => "bread",
            Self::Freezer => "frozen",
            Self::Other(_) => "",
        }
    }

    /// Whether the masthead needs to say which list this is.
    ///
    /// Bread is what a sheet is unless it says otherwise, and every sheet
    /// printed before this existed said nothing.
    pub fn names_itself(&self) -> bool {
        !matches!(self, Self::Bread)
    }

    /// The word the exporter used, for a filename or a pattern.
    pub fn word(&self) -> &str {
        match self {
            Self::Bread => "BREAD",
            Self::Freezer => "FREEZER",
            Self::Other(word) => word,
        }
    }
}

impl fmt::Display for Kind {
    /// The name a heading or a finding spells out.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bread => formatter.write_str("Bread"),
            Self::Freezer => formatter.write_str("Freezer"),
            Self::Other(word) => title_case(word, formatter),
        }
    }
}

/// `SOMETHING` -> `Something`, so an unknown kind reads like the two known
/// ones rather than shouting.
fn title_case(word: &str, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut characters = word.chars();
    let Some(first) = characters.next() else {
        return Ok(());
    };
    for upper in first.to_uppercase() {
        write!(formatter, "{upper}")?;
    }
    for lower in characters.flat_map(char::to_lowercase) {
        write!(formatter, "{lower}")?;
    }
    Ok(())
}
