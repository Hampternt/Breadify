//! The terminal side of the binary: what it prints, and what it does when
//! whoever asked for it stops listening.

use std::io::{ErrorKind, Write};
use std::process::Command;

use breadify::terminal;

fn breadify() -> Command {
    Command::new(env!("CARGO_BIN_EXE_breadify"))
}

#[test]
fn licences_says_what_is_embedded_and_under_what_terms() {
    let output = breadify().arg("licences").output().expect("runs");
    let said = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "{:?}", output.status);
    assert!(said.starts_with("Breadify "), "{said}");
    assert!(said.contains("MIT licensed"), "{said}");
    assert!(said.contains("SIL Open Font License 1.1"), "{said}");
    for family in ["Archivo", "Space Grotesk", "IBM Plex Mono"] {
        assert!(said.contains(family), "{family} is not listed:\n{said}");
    }
    assert!(
        said.contains("Matvare Expressen wordmark"),
        "the mark's own terms are not stated:\n{said}"
    );
}

#[test]
fn version_says_the_version_it_was_built_at() {
    let output = breadify().arg("--version").output().expect("runs");
    let said = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert_eq!(
        said.trim(),
        format!("breadify {}", env!("CARGO_PKG_VERSION"))
    );
}

/// Whoever reads the output is allowed to stop reading. `breadify licences |
/// head -1` closes the pipe, and Rust's runtime turns that into an `EPIPE` on
/// the next write rather than a signal — which `println!` used to answer with
/// a panic and a stack trace.
///
/// A pipe cannot be closed at a chosen instant from here, so the decision
/// itself is tested against a writer that simply says the pipe is gone.
#[test]
fn a_reader_that_stops_reading_is_not_an_error() {
    let mut gone = Refuses(ErrorKind::BrokenPipe);
    assert!(
        terminal::write(&mut gone, "anything at all").is_ok(),
        "a closed pipe is how a reader says it has enough, not a failure"
    );
}

/// But losing output to a real failure is its own bug, and must not be
/// swallowed along with it.
#[test]
fn a_write_that_actually_failed_still_says_so() {
    // Not `Interrupted`: `write_all` retries that one by contract, so a writer
    // that only ever returns it never returns at all.
    for kind in [ErrorKind::PermissionDenied, ErrorKind::StorageFull] {
        let mut refuses = Refuses(kind);
        let outcome = terminal::write(&mut refuses, "anything at all");
        assert_eq!(
            outcome.expect_err("should not be swallowed").kind(),
            kind,
            "{kind:?} is not a reader going away"
        );
    }
}

#[test]
fn what_is_written_arrives_whole() {
    let mut kept: Vec<u8> = Vec::new();
    terminal::write(&mut kept, "one\ntwo\nthree\n").expect("writes");
    assert_eq!(String::from_utf8_lossy(&kept), "one\ntwo\nthree\n");
}

/// A writer that will not write.
struct Refuses(ErrorKind);

impl Write for Refuses {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(self.0, "no"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(self.0, "no"))
    }
}
