//! Which list an export is, and what follows from it.

mod support;

use std::path::Path;

use breadify::list::Kind;
use support::{freezer_path, sample_path};

#[test]
fn the_samples_name_themselves() {
    assert_eq!(Kind::of(&sample_path()), Kind::Bread);
    assert_eq!(Kind::of(&freezer_path()), Kind::Freezer);
}

/// A file whose name says nothing is read as bread, which is what this app was
/// built for and what every export before the freezer one was.
#[test]
fn a_nameless_file_is_bread() {
    assert_eq!(Kind::of(Path::new("orders.xlsx")), Kind::Bread);
    assert_eq!(Kind::of(Path::new("")), Kind::Bread);
}

/// The exporter shouts its filenames; nobody should have to match its case.
#[test]
fn the_word_is_read_whatever_its_case() {
    assert_eq!(Kind::from_word("bread"), Kind::Bread);
    assert_eq!(Kind::from_word("Freezer"), Kind::Freezer);
    assert_eq!(Kind::from_word("FREEZER"), Kind::Freezer);
}

/// Crates are bread arithmetic. Anything that is not bread — including a kind
/// this app has never met — goes without rather than gets it wrong.
#[test]
fn only_bread_has_crates() {
    assert!(Kind::Bread.has_crates());
    assert!(!Kind::Freezer.has_crates());
    assert!(!Kind::Other("DRY".to_owned()).has_crates());
}

#[test]
fn every_kind_can_name_itself() {
    assert_eq!(Kind::Bread.to_string(), "Bread");
    assert_eq!(Kind::Freezer.to_string(), "Freezer");
    assert_eq!(Kind::Other("DRY-GOODS".to_owned()).to_string(), "Dry-goods");
    assert_eq!(Kind::Freezer.word(), "FREEZER");
    assert_eq!(Kind::Freezer.goods(), "frozen goods");
}
