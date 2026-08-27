//! What shapes a printed page.
//!
//! The printed form is fixed — six fields always print (decision D11) — so
//! this is a short list on purpose: whether the order id shows, how a refusal
//! to accept substitutes is marked, and the crate arithmetic.
//!
//! One field here is not the user's: which list the export holds. It sits with
//! the choices rather than beside them because it decides the same things they
//! do, and because a single channel for "what does this page show" is one that
//! cannot disagree with itself.

use crate::crates::CrateRules;
use crate::list::Kind;

/// How the no-substitutes state is drawn. The quiet state is the same in every
/// case; these are the three treatments the design offers for the loud one.
///
/// The words alone are the default. The design handoff argued for two
/// independent channels — badge and bar — so the loud state survives a
/// photocopy; the warehouse asked for the plainer page, and the words are
/// still set in the heading either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerTreatment {
    /// The words alone, in the heading.
    #[default]
    WordOnly,
    /// White on solid black, with a heavy bar down the block. Two channels, so
    /// it survives a photocopy.
    InvertedBadge,
    /// The bar alone.
    HeavyRule,
}

impl MarkerTreatment {
    pub const ALL: [Self; 3] = [Self::WordOnly, Self::InvertedBadge, Self::HeavyRule];

    pub fn label(self) -> &'static str {
        match self {
            Self::InvertedBadge => "Inverted badge",
            Self::HeavyRule => "Heavy left rule",
            Self::WordOnly => "Word only",
        }
    }

    /// Whether this treatment inverts the words into a filled badge.
    pub fn has_badge(self) -> bool {
        matches!(self, Self::InvertedBadge)
    }

    /// Whether this treatment runs a bar down the side of the block.
    pub fn has_rule(self) -> bool {
        matches!(self, Self::InvertedBadge | Self::HeavyRule)
    }
}

/// Everything that shapes a printed sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The one field that is the user's to show or hide.
    pub show_order_id: bool,
    pub marker: MarkerTreatment,
    pub crates: CrateRules,
    /// Which list this is — read off the export's filename, not chosen. Only
    /// a bread sheet counts crates; see [`Kind::has_crates`].
    pub list: Kind,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_order_id: true,
            marker: MarkerTreatment::default(),
            crates: CrateRules::default(),
            list: Kind::Bread,
        }
    }
}

impl Settings {
    /// The same settings, for a different list.
    pub fn for_list(mut self, list: Kind) -> Self {
        self.list = list;
        self
    }

    /// Whether this sheet draws crates at all.
    pub fn has_crates(&self) -> bool {
        self.list.has_crates()
    }
}

/// The fields that print whatever the user says, in the order the page sets
/// them.
pub const ALWAYS_PRINTED: [&str; 6] = [
    "Quantity",
    "Product Name",
    "Customer",
    "Department",
    "Accept alternatives",
    "Route nickname",
];

/// The field that is never printed, and why.
pub const NEVER_PRINTED: (&str, &str) = ("Route ordering", "the list order says it");
