//! The look commands' git argument lists (temper/docs/design/client/journey-writer.md, 7).

use std::{cell::RefCell, collections::BTreeSet, fmt::Write as _};

use crate::{
    host::{Ask, Step},
    look_compare, look_refresh, look_render,
};

// Each step checks its own rows. The names test uses the same declarations.
macro_rules! look_step_tests {
    ($(#[test] fn $name:ident() $body:block)*) => {
        $crate::look_git::look_step_tests! { @check $crate::look_git::check_lists;
            $(#[test] fn $name() $body)*
        }
    };
    (@check $check:path; $(#[test] fn $name:ident() $body:block)*) => {
        $(#[test] fn $name() {
            $check(stringify!($name), || $body);
        })*

        pub(crate) const GIT_TEST_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}
pub(crate) use look_step_tests;

type Arguments = Vec<Box<[u8]>>;

thread_local! {
    static LISTS: RefCell<Option<BTreeSet<Arguments>>> = const { RefCell::new(None) };
}

pub(crate) fn record(steps: &[Step]) {
    LISTS.with_borrow_mut(|lists| {
        for step in steps {
            if let Ask::Run(program) = &step.ask
                && program.program.as_ref() == b"git"
            {
                lists.as_mut().expect("git asks belong in a look_step_tests! block").insert(program.arguments.clone());
            }
        }
    });
}

// Each row is a test name followed by its git arguments, separated by tabs.
// Backslash escapes keep tabs and line endings within their fields.
const FIXTURE: &str = include_str!("look_git_lists.tsv");

pub(crate) fn check_lists(name: &str, run: impl FnOnce()) {
    LISTS.with_borrow_mut(|lists| *lists = Some(BTreeSet::new()));
    run();
    let lists = LISTS.with_borrow_mut(Option::take).expect("recorded git lists");
    let mut missing: BTreeSet<_> = lists.iter().map(|args| line(name, args)).collect();
    let surplus: Vec<_> = rows().filter(|row| test_name(row) == name && !missing.remove(*row)).collect();
    if !missing.is_empty() || !surplus.is_empty() {
        let mut changes = String::new();
        for row in missing {
            writeln!(changes, "add: {row:?}").expect("write to a string");
        }
        for row in surplus {
            writeln!(changes, "remove: {row:?}").expect("write to a string");
        }
        panic!("git lists for {name} in look_git_lists.tsv:\n{changes}");
    }
}

fn rows() -> impl Iterator<Item = &'static str> {
    FIXTURE.split_terminator('\n')
}

fn test_name(row: &str) -> &str {
    row.split_once('\t').map_or(row, |(name, _)| name)
}

#[test]
fn listed_git_lists_belong_to_look_step_tests() {
    let names: BTreeSet<_> =
        [look_compare::tests::GIT_TEST_NAMES, look_refresh::tests::GIT_TEST_NAMES, look_render::tests::GIT_TEST_NAMES]
            .into_iter()
            .flatten()
            .copied()
            .collect();
    let unknown: Vec<_> = rows().filter(|row| !names.contains(test_name(row))).collect();
    assert!(unknown.is_empty(), "remove lines in look_git_lists.tsv with no look step test: {unknown:#?}");
}

fn line(name: &str, args: &[Box<[u8]>]) -> String {
    let mut line = String::from(name);
    for arg in args {
        line.push('\t');
        for ch in std::str::from_utf8(arg).expect("listed git arguments are text").chars() {
            match ch {
                '\\' => line.push_str("\\\\"),
                '\t' => line.push_str("\\t"),
                '\r' => line.push_str("\\r"),
                '\n' => line.push_str("\\n"),
                _ => line.push(ch),
            }
        }
    }
    line
}
