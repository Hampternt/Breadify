//! Where the crate rules live between runs.
//!
//! How many slots a crate holds and how much room each bread takes are facts
//! about the bakery, not choices about today's print — somebody works them out
//! once, standing at the crates, and they should still be there tomorrow. The
//! rest of [`Settings`](crate::layout::Settings) is per-print and is not kept.
//!
//! The file is a few lines of text rather than JSON so that the person who set
//! the numbers can open it, read it and fix a typo without the app. Anything
//! it does not understand is ignored rather than refused: a settings file is
//! never worth failing a print over.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::crates::CrateRules;

const FILE: &str = "crates.conf";

const HEADER: &str = "\
# Breadify — how the bakery's crates work.
#
# Written by the app when you change something on the Configure step, and safe
# to edit by hand. Sizes are a percentage of one slot: 50 is a half, 200 takes
# the room of two. A bread that is not listed is a whole slot.
";

/// The file the rules are kept in, if this machine says where such things go.
pub fn path() -> Option<PathBuf> {
    Some(config_dir()?.join("breadify").join(FILE))
}

/// The rules as last saved, or nothing if they have never been saved or the
/// file cannot be read.
pub fn load() -> Option<CrateRules> {
    let text = std::fs::read_to_string(path()?).ok()?;
    Some(read(&text))
}

/// Writes the rules, keeping the bread names as comments so the file reads.
///
/// `names` is what the open export knows; anything already in the file keeps
/// the name it had, so a size set from Monday's export is still legible after
/// Tuesday's is saved over it.
pub fn save(rules: &CrateRules, names: &BTreeMap<u32, String>) -> Result<PathBuf, String> {
    let target = path().ok_or("this machine does not say where settings go")?;
    let directory = target
        .parent()
        .ok_or("the settings path has no directory")?;

    let mut labels = std::fs::read_to_string(&target)
        .map(|text| read_labels(&text))
        .unwrap_or_default();
    labels.extend(names.iter().map(|(id, name)| (*id, name.clone())));

    std::fs::create_dir_all(directory)
        .map_err(|error| format!("could not make {}: {error}", directory.display()))?;

    // Write beside it and rename, so a crash halfway through leaves the old
    // rules rather than half of the new ones.
    let scratch = target.with_extension("writing");
    std::fs::write(&scratch, render(rules, &labels))
        .map_err(|error| format!("could not write {}: {error}", scratch.display()))?;
    std::fs::rename(&scratch, &target)
        .map_err(|error| format!("could not replace {}: {error}", target.display()))?;

    Ok(target)
}

/// The rules a settings file describes. Lines it does not understand are
/// skipped, and anything missing keeps its default.
pub fn read(text: &str) -> CrateRules {
    let mut rules = CrateRules::default();

    for line in text.lines() {
        let mut words = line.split_whitespace();
        match (words.next(), words.next()) {
            (Some("large"), Some(slots)) => {
                if let Ok(slots) = slots.parse() {
                    rules.large_capacity = slots;
                }
            }
            (Some("small"), Some(slots)) => {
                if let Ok(slots) = slots.parse() {
                    rules.small_capacity = slots;
                }
            }
            (Some("size"), Some(id)) => {
                if let (Ok(id), Some(Ok(percent))) = (id.parse(), words.next().map(str::parse)) {
                    rules.set_size(id, percent);
                }
            }
            _ => {}
        }
    }
    rules
}

/// The bread names a settings file carries, so re-saving does not throw away
/// the label of a bread that is not in today's export.
fn read_labels(text: &str) -> BTreeMap<u32, String> {
    text.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("size ")?;
            let mut words = rest.split_whitespace();
            let id: u32 = words.next()?.parse().ok()?;
            let _percent = words.next()?;
            let name = words.collect::<Vec<_>>().join(" ");
            (!name.is_empty()).then_some((id, name))
        })
        .collect()
}

/// The file's whole text. Deterministic — the sizes are in product order, so
/// saving twice without changing anything produces the same bytes.
pub fn render(rules: &CrateRules, labels: &BTreeMap<u32, String>) -> String {
    let mut out = String::from(HEADER);
    out.push_str(&format!(
        "\nlarge {}\nsmall {}\n",
        rules.large_capacity, rules.small_capacity
    ));

    let sizes: BTreeMap<u32, u32> = rules
        .size_percent
        .iter()
        .map(|(id, percent)| (*id, *percent))
        .collect();
    if sizes.is_empty() {
        out.push_str("\n# Every bread is a whole slot.\n");
        return out;
    }

    out.push_str("\n# size <product id> <percent of one slot>   <name, for you not the app>\n");
    for (id, percent) in sizes {
        match labels.get(&id) {
            Some(name) => out.push_str(&format!("size {id} {percent}   {name}\n")),
            None => out.push_str(&format!("size {id} {percent}\n")),
        }
    }
    out
}

/// Where this platform keeps a program's settings.
fn config_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        return std::env::var_os("APPDATA").map(PathBuf::from);
    }

    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
}
