//! One table test's rows (skein/docs/design/testing.md, 6): cases of one
//! behaviour share a process (skein/docs/foundation/testing-strategy.md,
//! 8), and every row still runs and names itself when it fails.
//!
//! Ordinary Rust (programming-model.md, 10.2): it runs in tests only.

use std::collections::BTreeSet;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Runs `body` on every row, then panics naming each row that panicked,
/// with its message. Fewer than two rows, or two rows of one name, panic
/// before any row runs.
#[track_caller]
pub fn check<R>(rows: impl IntoIterator<Item = (&'static str, R)>, body: impl Fn(R)) {
    let rows: Vec<_> = rows.into_iter().collect();
    assert!(rows.len() >= 2, "a table needs at least two rows");
    let mut names = BTreeSet::new();
    for (name, _) in &rows {
        assert!(names.insert(name), "two rows have one name: {name}");
    }
    let total = rows.len();
    let mut failures = Vec::new();
    for (name, row) in rows {
        if let Err(payload) = catch_unwind(AssertUnwindSafe(|| body(row))) {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("<no message>");
            failures.push(format!("{name}: {message}"));
        }
    }
    assert!(failures.is_empty(), "{} of {total} rows failed\n{}", failures.len(), failures.join("\n"));
}

#[cfg(test)]
mod tests;
