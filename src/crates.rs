//! How many crates an order needs.
//!
//! Crates come in two sizes and the rule is fewest containers. Breads are not
//! all the same size, so each one carries a size modifier — a percentage of a
//! standard slot — and the arithmetic runs on slots rather than raw loaves
//! (decision D17). With every modifier left at 100 % it reduces exactly to
//! counting loaves.

use std::collections::HashMap;

use crate::order::Order;

/// A bread that takes exactly one slot. Half-size items are `50`, bulky ones
/// above `100`.
pub const STANDARD_SIZE: u32 = 100;

/// The sizes worth a button, as the fraction of a slot a driver would say out
/// loud and the percentage the arithmetic runs on.
///
/// Thirds round down — three at `33` come to 99 hundredths and fill one slot,
/// which is right; at `34` they would ask for two.
pub const SIZE_PRESETS: [(&str, u32); 8] = [
    ("1/4", 25),
    ("1/3", 33),
    ("1/2", 50),
    ("2/3", 66),
    ("1", STANDARD_SIZE),
    ("1 1/2", 150),
    ("2", 200),
    ("3", 300),
];

/// The crate sizes, and how much room each bread takes in one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateRules {
    /// Slots in a large crate.
    pub large_capacity: u32,
    /// Slots in a small crate. Smaller than [`large_capacity`](Self::large_capacity).
    pub small_capacity: u32,
    /// Per-product size, as a percentage of one slot. Anything absent is a
    /// standard bread.
    pub size_percent: HashMap<u32, u32>,
}

impl Default for CrateRules {
    /// Crates of ten and five, every bread a standard size — the arithmetic
    /// the docs' worked examples were computed with.
    fn default() -> Self {
        Self {
            large_capacity: 10,
            small_capacity: 5,
            size_percent: HashMap::new(),
        }
    }
}

impl CrateRules {
    /// How much room one of `product_id` takes, as a percentage of a slot.
    pub fn size_of(&self, product_id: u32) -> u32 {
        self.size_percent
            .get(&product_id)
            .copied()
            .unwrap_or(STANDARD_SIZE)
    }

    /// Marks a bread as taking `percent` of a slot: `50` for a roll, `200` for
    /// something that takes the room of two.
    ///
    /// Setting a bread back to a whole slot forgets it rather than recording
    /// the default, so "which breads has someone had to say something about"
    /// stays answerable — and so setting a standard bread to standard is not a
    /// change the rest of the app has to react to.
    pub fn set_size(&mut self, product_id: u32, percent: u32) {
        if percent == STANDARD_SIZE {
            self.size_percent.remove(&product_id);
            return;
        }
        self.size_percent.insert(product_id, percent);
    }

    /// Whether someone has said anything about this bread.
    pub fn is_custom(&self, product_id: u32) -> bool {
        self.size_percent.contains_key(&product_id)
    }
}

/// How a size reads: the fraction if it is one of the buttons, and the bare
/// percentage if someone typed something else.
pub fn spoken(percent: u32) -> String {
    SIZE_PRESETS
        .iter()
        .find(|(_, value)| *value == percent)
        .map_or_else(|| format!("{percent} %"), |(label, _)| (*label).to_owned())
}

/// The crates one order needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CrateCount {
    pub large: u32,
    pub small: u32,
}

impl CrateCount {
    /// Crates to carry, of either size — what the glyphs beside a customer
    /// name add up to.
    pub fn total(&self) -> u32 {
        self.large + self.small
    }
}

/// Slots an order fills, rounding a part-slot up to a whole one.
///
/// A bread at 50 % and a quantity of 3 fills two slots, not one and a half:
/// half a slot still occupies a slot's worth of crate.
pub fn slots(order: &Order, rules: &CrateRules) -> u32 {
    let hundredths: u64 = order
        .lines
        .iter()
        .map(|line| u64::from(line.quantity) * u64::from(rules.size_of(line.product.id)))
        .sum();

    let slots = hundredths.div_ceil(u64::from(STANDARD_SIZE));
    u32::try_from(slots).unwrap_or(u32::MAX)
}

/// How many crates of each size an order needs, in the fewest containers.
///
/// A remainder that fits a small crate takes one; a remainder too big for one
/// takes a large crate rather than two smalls.
pub fn count(order: &Order, rules: &CrateRules) -> CrateCount {
    let capacity = rules.large_capacity.max(1);
    let small_capacity = rules.small_capacity.min(capacity.saturating_sub(1)).max(1);

    let slots = slots(order, rules);
    let large = slots / capacity;
    let remainder = slots % capacity;

    match remainder {
        0 => CrateCount { large, small: 0 },
        _ if remainder <= small_capacity => CrateCount { large, small: 1 },
        _ => CrateCount {
            large: large + 1,
            small: 0,
        },
    }
}
