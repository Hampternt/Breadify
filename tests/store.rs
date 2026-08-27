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

/// A scratch directory of this test's own, so the suite never touches the
/// settings of whoever is running it.
fn scratch(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join("breadify-store").join(name);
    let _ = std::fs::remove_dir_all(&directory);
    directory.join("crates.conf")
}

#[test]
fn saving_makes_the_directory_and_the_file_reads_back() {
    let file = scratch("fresh");
    assert!(!file.exists(), "the test starts with nothing");

    let written = store::save_to(&file, &rules(), &labels()).expect("saves");

    assert_eq!(written, file);
    assert!(file.exists(), "and the directory under it was made");
    assert_eq!(store::load_from(&file), Some(rules()));
}

#[test]
fn saving_over_a_file_keeps_the_names_the_caller_has_forgotten() {
    let file = scratch("names");
    store::save_to(&file, &rules(), &labels()).expect("saves");

    // Tomorrow's export has neither bread in it, so the app knows no names.
    store::save_to(&file, &rules(), &BTreeMap::new()).expect("saves again");

    let text = std::fs::read_to_string(&file).expect("reads");
    for name in labels().values() {
        assert!(text.contains(name.as_str()), "{name} was lost:\n{text}");
    }
    assert_eq!(store::load_from(&file), Some(rules()));
}

#[test]
fn a_save_leaves_no_scratch_file_behind() {
    let file = scratch("tidy");
    store::save_to(&file, &rules(), &labels()).expect("saves");

    let left: Vec<String> = std::fs::read_dir(file.parent().expect("a directory"))
        .expect("reads the directory")
        .filter_map(|entry| Some(entry.ok()?.file_name().to_string_lossy().into_owned()))
        .collect();

    assert_eq!(left, ["crates.conf"], "the rename cleans up after itself");
}

#[test]
fn a_later_save_replaces_the_earlier_one_rather_than_merging() {
    let file = scratch("replace");
    store::save_to(&file, &rules(), &labels()).expect("saves");

    let mut fewer = CrateRules::default();
    fewer.set_size(7, 25);
    store::save_to(&file, &fewer, &labels()).expect("saves again");

    let read = store::load_from(&file).expect("reads");
    assert_eq!(read, fewer);
    assert_eq!(read.size_of(7), 25, "the changed bread changed");
    assert!(!read.is_custom(9), "the removed bread is gone, not stale");
    assert_eq!(read.large_capacity, CrateRules::default().large_capacity);
}

#[test]
fn saving_somewhere_it_cannot_write_says_so_and_does_not_panic() {
    // A path whose parent is an existing *file* — the directory cannot be made.
    let blocker = scratch("blocked");
    store::save_to(&blocker, &CrateRules::default(), &BTreeMap::new()).expect("saves");

    let impossible = blocker.join("crates.conf");
    let outcome = store::save_to(&impossible, &rules(), &BTreeMap::new());

    assert!(outcome.is_err(), "{outcome:?}");
    assert_eq!(
        store::load_from(&blocker).as_ref(),
        Some(&CrateRules::default()),
        "and the rules that were already there are untouched"
    );
}
