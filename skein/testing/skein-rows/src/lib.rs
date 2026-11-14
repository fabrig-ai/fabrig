//! One table test's rows (skein/docs/design/testing/README.md, 6): cases of one
//! behaviour share a process (skein/docs/foundation/testing-strategy.md,
//! 8). Each row is named before it runs; the first failure ends the table.
//!
//! Ordinary Rust (programming-model.md, 10.2): it runs in tests only.

use std::collections::BTreeSet;

/// Runs `body` on each row in order, naming it on stderr before it runs.
/// The first failure ends the table. Fewer than two rows, or two rows of
/// one name, panic before any row runs.
#[track_caller]
#[expect(clippy::print_stderr, reason = "the row's name is the table's report (testing/README.md, 6)")]
pub fn check<R>(rows: impl IntoIterator<Item = (&'static str, R)>, body: impl Fn(R)) {
    check_with_report(rows, body, |line| eprintln!("{line}"));
}

#[track_caller]
fn check_with_report<R>(rows: impl IntoIterator<Item = (&'static str, R)>, body: impl Fn(R), report: impl Fn(&str)) {
    let rows: Vec<_> = rows.into_iter().collect();
    assert!(rows.len() >= 2, "a table needs at least two rows");
    let mut names = BTreeSet::new();
    for (name, _) in &rows {
        assert!(names.insert(name), "two rows have one name: {name}");
    }
    for (name, row) in rows {
        report(&format!("skein-rows: row {name}"));
        body(row);
    }
}

#[cfg(test)]
mod tests;
