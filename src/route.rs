//! Grouping orders into routes, and putting both into printing order.
//!
//! Two sorts, and the first is the one most likely to go wrong: route
//! nicknames are text even when they look like numbers, so sorting them as
//! text gives `1, 10, 11, … 2`. See `docs/print-spec.md` §2.

use std::collections::BTreeMap;

use crate::order::Order;

/// One route's worth of work, with its stops already in delivery order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// Exactly as the file spells it — `7`, `hau 2` (decision D12).
    pub nickname: String,
    /// Sequenced stops first in delivery order, then the unsequenced ones.
    pub stops: Vec<Order>,
}

impl Route {
    /// Stops the export gave no position to. They print after the rest, under
    /// a flag, because nobody chose to put them last.
    pub fn unsequenced(&self) -> impl Iterator<Item = &Order> {
        self.stops.iter().filter(|stop| !stop.is_sequenced())
    }

    /// Every order line on the route, which is what fills the page.
    pub fn line_count(&self) -> usize {
        self.stops.iter().map(|stop| stop.lines.len()).sum()
    }
}

/// Groups orders into routes, both in printing order.
///
/// Routes come out in natural order and stops in delivery order; see
/// [`natural_key`] and [`printing_position`].
pub fn group(orders: Vec<Order>) -> Vec<Route> {
    let mut by_nickname: BTreeMap<String, Vec<Order>> = BTreeMap::new();
    for order in orders {
        by_nickname
            .entry(order.route.clone())
            .or_default()
            .push(order);
    }

    let mut routes: Vec<Route> = by_nickname
        .into_iter()
        .map(|(nickname, mut stops)| {
            sort_stops(&mut stops);
            Route { nickname, stops }
        })
        .collect();

    routes.sort_by(|left, right| natural_key(&left.nickname).cmp(&natural_key(&right.nickname)));
    routes
}

/// Puts a route's stops into the order a driver drives them.
pub fn sort_stops(stops: &mut [Order]) {
    stops.sort_by(|left, right| printing_position(left).cmp(&printing_position(right)));
}

/// What decides where a stop prints: sequenced stops in ascending sequence,
/// then the unsequenced ones, with address, department and order id breaking
/// ties so two runs of one file print identically.
///
/// Equal sequences are legitimate — one site with several delivery points —
/// which is exactly why the tiebreak is not optional.
fn printing_position(stop: &Order) -> (bool, u32, &str, Option<&str>, i64) {
    (
        !stop.is_sequenced(),
        stop.sequence,
        &stop.delivery_street,
        stop.department.as_deref(),
        stop.id,
    )
}

/// How a route nickname sorts: by its leading number where it has one,
/// otherwise by name and then by the number that follows it.
///
/// `1, 2, … 14, hau 1, hau 2` — never `1, 10, 11, … 2`.
pub fn natural_key(nickname: &str) -> RouteKey<'_> {
    let digits = nickname
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(nickname.len());
    if digits > 0
        && let Ok(number) = nickname[..digits].parse()
    {
        return RouteKey::Numbered(number, &nickname[digits..]);
    }

    let (name, number) = split_trailing_number(nickname);
    RouteKey::Named(name, number)
}

/// The sortable shape of a route nickname. Numbered routes come first,
/// because every route that is only a number belongs to the home town and the
/// named ones are elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RouteKey<'a> {
    /// `14` -> `Numbered(14, "")`.
    Numbered(u64, &'a str),
    /// `hau 2` -> `Named("hau", 2)`.
    Named(&'a str, u64),
}

/// `hau 2` -> `("hau", 2)`; a name with no trailing number keeps a zero.
fn split_trailing_number(nickname: &str) -> (&str, u64) {
    let trimmed = nickname.trim_end();
    let digits_start = trimmed
        .rfind(|character: char| !character.is_ascii_digit())
        .map_or(0, |last| last + 1);

    let Ok(number) = trimmed[digits_start..].parse() else {
        return (trimmed, 0);
    };
    (trimmed[..digits_start].trim_end(), number)
}
