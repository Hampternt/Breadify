//! The two bakeries, as the printed page refers to them.
//!
//! The export spells a supplier in lower case (`sandnes bakeri`) and again as
//! a pick position (`X-Sandnes Bakeri`); the page uses a two-letter code on
//! every bread line, because spelling the name out on each row is what would
//! cost the body text its size. Decision D4.

/// The bakeries this app knows: the export's spelling, the code the page
/// uses, and the name a heading spells out. The order is the order their
/// columns print in.
pub const KNOWN: [(&str, &str, &str); 2] = [
    ("sandnes bakeri", "SB", "Sandnes Bakeri"),
    ("bakehuset", "BH", "Bakehuset"),
];

/// The two-letter code for a bread line.
///
/// A bakery nobody has configured yet falls back to its initials, or its
/// first two letters when it is a single word.
pub fn code(supplier: &str) -> String {
    if let Some((_, code, _)) = known(supplier) {
        return (*code).to_owned();
    }

    let initials: String = supplier
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect();
    let derived = if initials.chars().count() >= 2 {
        initials
    } else {
        supplier.chars().take(2).collect()
    };
    derived.to_uppercase()
}

/// The name spelled out, for the legend and the route total.
pub fn display_name(supplier: &str) -> String {
    if let Some((_, _, name)) = known(supplier) {
        return (*name).to_owned();
    }
    title_case(supplier)
}

/// Where a supplier's column sits: the house order first, then anything new,
/// alphabetically.
pub fn column_position(supplier: &str) -> (usize, String) {
    let rank = KNOWN
        .iter()
        .position(|(name, _, _)| name.eq_ignore_ascii_case(supplier));
    (rank.unwrap_or(KNOWN.len()), supplier.to_lowercase())
}

fn known(supplier: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    KNOWN
        .iter()
        .find(|(name, _, _)| name.eq_ignore_ascii_case(supplier))
}

fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
