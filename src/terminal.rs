//! Writing to a terminal that is allowed to stop listening.
//!
//! `breadify licences | head -1` closes the pipe on purpose, and every Unix
//! tool stops quietly when that happens. Rust's runtime ignores `SIGPIPE`, so
//! the write comes back `EPIPE` instead — and `println!`, which unwraps,
//! answers with a panic and a stack trace where nothing went wrong. Windows
//! has no `SIGPIPE` and reaches the same error by its own route, so this is
//! handled here rather than by restoring a signal handler.
//!
//! A real failure — a full disk, a revoked handle — still comes back, because
//! silently losing output the user asked for is its own bug.

use std::io::{ErrorKind, Result, Write};

/// Writes `text` in one go, and says nothing happened if the reader has gone.
///
/// One write rather than one per line: it is fewer syscalls, and the output of
/// a short command arrives whole or not at all.
pub fn write(out: &mut impl Write, text: &str) -> Result<()> {
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Err(error) if error.kind() == ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}
