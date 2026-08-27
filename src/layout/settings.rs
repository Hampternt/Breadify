//! What the user gets to change about the printed page.
//!
//! The printed form is fixed — six fields always print (decision D11) — so
//! this is a short list on purpose: whether the order id shows, how a refusal
//! to accept substitutes is marked, and the crate arithmetic.

use crate::crates::CrateRules;

/// How the no-substitutes state is drawn. The quiet state is the same in every
/// case; these are the three treatments the design offers for the loud one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerTreatment {
    /// White on solid black, with a heavy bar down the block. Two channels, so
    /// it survives a photocopy.
    #[default]
    InvertedBadge,
    /// The bar alone.
    HeavyRule,
    /// The words alone, in the heading.
    WordOnly,
}

impl MarkerTreatment {
    pub const ALL: [Self; 3] = [Self::InvertedBadge, Self::HeavyRule, Self::WordOnly];

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

/// Everything the user can change about a printed sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The one field that is the user's to show or hide.
    pub show_order_id: bool,
    pub marker: MarkerTreatment,
    pub crates: CrateRules,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_order_id: true,
            marker: MarkerTreatment::default(),
            crates: CrateRules::default(),
        }
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
