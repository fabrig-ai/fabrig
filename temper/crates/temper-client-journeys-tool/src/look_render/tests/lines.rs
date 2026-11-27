//! Listed stdout from the look-render step tests (temper/docs/design/client/journey-writer.md, 7).

use std::{cell::RefCell, collections::BTreeMap, fmt::Write as _};

use super::GIT_TEST_NAMES;

// The test name, a tab, and one line as look-render writes it.
const FIXTURE: &str = include_str!("../../look_render_lines.tsv");

thread_local! {
    static PRINTED: RefCell<String> = const { RefCell::new(String::new()) };
}

pub(super) fn record(stdout: &str) {
    PRINTED.with_borrow_mut(|printed| printed.push_str(stdout));
}

pub(super) fn check(name: &str, run: impl FnOnce()) {
    run();
    let printed = PRINTED.with_borrow_mut(std::mem::take);
    let mut printed = printed.split_inclusive('\n');
    let mut listed = rows().filter_map(|(test, line)| (test == name).then_some(line));
    loop {
        let (seen, expected) = (printed.next(), listed.next());
        if seen != expected {
            let mut changes = String::new();
            for (action, lines) in [
                ("remove", expected.into_iter().chain(listed).collect::<Vec<_>>()),
                ("add", seen.into_iter().chain(printed).collect()),
            ] {
                for line in lines {
                    let line = line.strip_suffix('\n').unwrap_or(line);
                    let row = format!("{name}\t{line}");
                    writeln!(changes, "{action}: {row:?}").expect("write to a string");
                }
            }
            panic!("stdout for {name} in look_render_lines.tsv:\n{changes}");
        }
        if seen.is_none() {
            break;
        }
    }
}

fn rows() -> impl Iterator<Item = (&'static str, &'static str)> {
    FIXTURE.split_inclusive('\n').map(|row| row.split_once('\t').expect("listed stdout has a test name and a line"))
}

#[test]
fn listed_render_lines_belong_to_look_render_step_tests() {
    let unknown: Vec<_> = rows().filter(|(name, _)| !GIT_TEST_NAMES.contains(name)).collect();
    assert!(unknown.is_empty(), "remove lines in look_render_lines.tsv with no look-render step test: {unknown:#?}");
}

pub(crate) fn outputs() -> impl Iterator<Item = (&'static str, String)> {
    let mut by_test = BTreeMap::<_, Vec<_>>::new();
    for (name, line) in rows() {
        by_test.entry(name).or_default().push(line);
    }
    by_test.into_iter().flat_map(|(name, lines)| {
        let mut stdout = String::new();
        let mut outputs = Vec::new();
        for line in lines {
            stdout.push_str(line);
            if line.starts_with("result\t") {
                outputs.push((name, std::mem::take(&mut stdout)));
            }
        }
        assert!(stdout.is_empty(), "{name}: lines after the last result: {stdout:?}");
        outputs
    })
}
