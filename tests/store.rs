//! The settings file: what it says, and what it survives.

use std::collections::BTreeMap;

use breadify::crates::{CrateRules, STANDARD_SIZE};
use breadify::store;

fn labels() -> BTreeMap<u32, String> {
    [
        (7, "Ryfylkebrød Sandnes Oppdelt Bakeri".to_owned()),
        (9, "Emballerte Skoleboller 2 Pk Sandnes Bakeri".to_owned()),
    ]
    .into_iter()
    .collect()
}

fn rules() -> CrateRules {
    let mut rules = CrateRules {
        large_capacity: 12,
        small_capacity: 6,
        ..CrateRules::default()
    };
    rules.set_size(7, 50);
    rules.set_size(9, 200);
    rules
}

#[test]
fn what_is_written_is_what_comes_back() {
    let written = store::render(&rules(), &labels());
    assert_eq!(store::read(&written), rules());
}

#[test]
fn saving_twice_without_changing_anything_writes_the_same_bytes() {
    assert_eq!(
        store::render(&rules(), &labels()),
        store::render(&rules(), &labels()),
        "a HashMap's order must not reach the file"
    );
}

#[test]
fn the_names_are_there_for_the_reader_and_ignored_by_the_app() {
    let written = store::render(&rules(), &labels());
    assert!(
        written.contains("Ryfylkebrød Sandnes Oppdelt Bakeri"),
        "a person opening the file can tell which bread is which:\n{written}"
    );

    // The same rules with the names stripped still read back identically.
    let nameless = store::render(&rules(), &BTreeMap::new());
    assert_eq!(store::read(&nameless), store::read(&written));
}

#[test]
fn a_bread_at_a_whole_slot_is_not_written_at_all() {
    let mut rules = CrateRules::default();
    rules.set_size(7, STANDARD_SIZE);
    let written = store::render(&rules, &labels());

    assert!(!written.contains("size 7"), "{written}");
    assert_eq!(store::read(&written), CrateRules::default());
}

#[test]
fn a_file_someone_has_edited_badly_is_read_for_what_it_has() {
    let text = "\
# a comment
large 8
small
small four
size 7 50   Ryfylkebrød
size nine 25
size 11
size 12 abc
nonsense line here

size 13 300
";
    let read = store::read(text);

    assert_eq!(read.large_capacity, 8, "the line that parsed took effect");
    assert_eq!(
        read.small_capacity,
        CrateRules::default().small_capacity,
        "the lines that did not keep the default"
    );
    assert_eq!(read.size_of(7), 50);
    assert_eq!(read.size_of(13), 300);
    assert_eq!(read.size_percent.len(), 2, "and nothing else got through");
}

#[test]
fn an_empty_file_is_the_defaults() {
    assert_eq!(store::read(""), CrateRules::default());
    assert_eq!(
        store::read("# nothing but a comment\n"),
        CrateRules::default()
    );
}

#[test]
fn the_rules_go_somewhere_a_person_could_find() {
    let path = store::path().expect("this machine says where settings go");
    assert!(path.is_absolute(), "{}", path.display());
    assert!(path.ends_with("breadify/crates.conf"), "{}", path.display());
}
