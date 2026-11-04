+++
id = "notes-drop-entries"
part = "closed-project"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-notes/src/drop_project.rs", "jig/crates/jig-domain-notes/src/tests.rs", "jig/tests/notes/src/referee.rs", "jig/tests/notes/tests/suite/drop_project.rs", "jig/tests/core/tests/suite/deletion.rs"]
effort = "medium"
minutes = 45
+++

# Deleting a project erases its notes' entries

## Goal

A project's deletion erases its notes (`jig/docs/design/domain/people.md`, 5.2, "Then its own rows"),
but `drop_project` erases only index lines: every `Entry`, with its body, stays in the store for
good. No read reaches it, so this is a retention leak on disk. An entry has one scope
(`jig/docs/design/domain/engine.md`, 10) and `save_entry` writes it one line, so the header comment's
"may also be indexed outside the project" is false outside one test's seed.

Acceptance:

- Each line a project drop erases erases its `Key::Entry { name }` in the same decision.
- `a_deleted_project_leaves_only_its_spend` (core world) counts a project's entries as its rows, and
  fails on the old code.
- The notes world's referee admits those erases, and the store after a drop holds no entry of the
  project.
- Entries left by earlier deletions stay: there is no scan, and this plan adds none.
- Every test of `jig-domain-notes`, `jig-notes-world`, `jig-domain-core` and `jig-core-world` stays
  green.

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `jig/docs/design/domain/people.md`, 5.2; `jig/docs/design/domain/engine.md`, 10.
- The code: `jig/crates/jig-domain-notes/src/drop_project.rs`, `save_entry` in `src/domain.rs`.
- The worlds: `jig/tests/notes/src/referee.rs`, `jig/tests/notes/tests/suite/drop_project.rs`,
  `jig/tests/core/tests/suite/deletion.rs` (`rows_naming`, `scope_names`).

## Owns and may change

- `jig/crates/jig-domain-notes/src/drop_project.rs`: `loaded`'s `Record::Line` arm and the module
  comment, which says the entries go with their lines.
- `jig/crates/jig-domain-notes/src/tests.rs`: `a_project_drop_retries_busy_loads_and_answers_only_after_an_empty_page`
  matches the line's erase, then the entry's, then the next load, and asserts the entry's key.
- `jig/tests/notes/src/referee.rs`: `before_commit`'s `DropProject` branch admits `Key::Entry` erases
  of an entry in the project's scope; its panic text.
- `jig/tests/notes/tests/suite/drop_project.rs`: the module comment; `in_project` counts a
  `Key::Entry` by the stored entry's scope; the test `dropping_a_project_erases_its_scopes_lines_and_retains_entries`
  renamed `dropping_a_project_erases_its_scopes_lines_and_entries`, its commit-count message; delete
  `an_entry_shared_with_another_scope_keeps_that_line`, which seeds a state no write makes.
- `jig/tests/core/tests/suite/deletion.rs`: `rows_naming`'s `notes::Record::Entry` arm and its comment.
- `jig/tests/notes/tests/suite/fuzzy/simulation.rs` and `jig/tests/notes/src/world.rs`: if their
  checks assume a drop keeps entries, update those assumptions and say so.
- Options: if the commit count in `drop_project.rs` changes, the pages are the commits, so a change
  means something else moved: stop and ask. If `jig-domain-core`'s tests count a drop's erases,
  update the count and say so.

## Steps

1. In `rows_naming`: `notes::Record::Entry(row) => scope_names(&row.scope, project)`. See
   `a_deleted_project_leaves_only_its_spend` fail, naming the entries.
2. In `loaded`, keep `line.name` before the line's key moves, and after the line's erase push
   `Request::Erase { key: Key::Entry { name } }`.
3. The referee and the notes tests, as above.
4. `heavy cargo nextest run -p jig-core-world --test suite -E 'test(=deletion::a_deleted_project_leaves_only_its_spend)'`,
   then `heavy cargo nextest run -p jig-domain-notes -p jig-notes-world -p jig-domain-core -p jig-core-world`.
5. `cargo fmt`; old forms; `heavy cargo clippy -p jig-domain-notes -p jig-notes-world -p jig-core-world --all-targets -- -D warnings`;
   commit and push.

## Report

As the implementer's text asks, with how the test failed on the old code.

## Answers

2026-11-04. Extend the task to the project-drop unit expectation in
`jig/crates/jig-domain-notes/src/tests.rs`, or hand it to another task? Extend: the acceptance's
four packages cannot be green without it, and the fix itself is what turns it red, so the file is
now in this spec's `files` and "Owns and may change". Only that test changes: match
`[Erase { key }, Erase { key: entry }, Load { .. }]`, assert
`entry == &crate::Key::Entry { name: 20 }`, and keep every other assertion, the `key` cursor checks
included. The pushed commit `84908ab7f7` on `bugs-jig/notes-drop-entries` stands: add to it, run the
failing test alone, then step 4's four packages, and finish step 5.
