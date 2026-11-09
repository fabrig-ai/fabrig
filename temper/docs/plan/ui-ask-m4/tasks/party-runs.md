+++
id = "party-runs"
part = "party"
kind = "feature"
needs = ["core-surface"]
files = ["jig/crates/jig-domain-core/src/routing/party.rs", "jig/crates/jig-domain-notes/src/domain.rs", "jig/crates/jig-domain-notes/src/boundary.rs", "jig/crates/jig-domain-notes/src/tests.rs", "jig/tests/core/tests/suite/party_runs.rs", "jig/tests/notes/tests/suite/simulation.rs"]
effort = "high"
minutes = 180
+++

## Goal

Let the runs that explain to their person write and drop that person's
preferences, and hold the scope's bounds in the notes child
(`jig/docs/design/domain/engine.md`, 10.1, *Its entries are preferences*
and *Written for its party*): fill `routing/party.rs`, which
`core-surface` made and wired.

Acceptance:

- `routing/party.rs` admits a run's `NamedAction::Note` and `NoteDelete` in
  `Scope::Party { party }` exactly when the run's task's requester is
  `Party::Person(party)` and its authority holds the asking family
  (`authority.md`, section 3); every other run's write or drop there is
  refused beyond authority naming the scope; `NoteLines` in a party's
  scope is refused as a body is (below); a `NoteDelete` of any other scope
  stays refused beyond authority. An admitted call is decided and recorded
  as `Note` is, answered from its record when asked again under the same
  name; a drop goes to the notes child's existing deletion at the revision
  named, author the run's task and attempt, answered
  `CallPart::NoteDeleted { name }` through `core-surface`'s success path.
- The notes child (`domain.rs`, `valid_new`, `valid_entry`, `valid_change`,
  `loaded`) holds a party's scope, for every writer, a party's correction
  included: a description past `party_bytes` refused
  `Refusal::PastBound { bytes, bound: party_bytes }`; a non-empty body
  refused `Refusal::PastBound { bytes, bound: 0 }`; a new entry past
  `party_entries` refused `Refusal::Full`. `PastBound`'s doc comment says it
  bounds a party's line as well as a body.
- `jig/tests/core/tests/suite/party_runs.rs`, beside the settings run's
  fixtures (`jig/tests/core/tests/settings/support.rs`), one table story,
  `a_run_keeps_its_persons_preferences_and_no_one_elses`: the settings run,
  its task requested by person P and given the asking family, adds a line,
  revises it, drops it, each answered and its store rows as expected, and
  asked again under a call's name answered from the record; the same run
  writing person Q's scope, a run without the asking family, and a run
  whose task a task requested, each refused beyond authority with the
  store unchanged; a thirteenth line `Full`, a line of 121 bytes and a body
  `PastBound`.
- Rows in `jig-domain-notes/src/tests.rs` for the three bounds and a
  party's correction past them; jig's notes world's simulation
  (`jig/tests/notes/tests/suite/simulation.rs`) draws a party's scope among
  its scopes.
- Delete the **Not yet** entries `10.1, *Its entries are preferences* and
  *Written for its party*` in `jig/docs/design/domain/engine.status.md` and
  `3, *Notes*` in `jig/docs/design/domain/authority.status.md`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  sections 1 and 2.
- `jig/docs/design/domain/engine.md`, 10.1, *Its entries are preferences*,
  *Written for its party*; section 10, *Written by a tool*;
  `jig/docs/design/domain/authority.md`, section 3, *Notes*.
- `core-surface`'s report on the trunk (its branch's last commit message
  and `routing/party.rs`' module doc): the stub's functions and call sites.
- Patterns: `jig-domain-core/src/routing/mod.rs:158` (`authorize_tool`) and
  `:271` (`note_readable`); `routing/run/calls.rs:714`; the deletion chain,
  `routing/effect/note.rs:245`, `jig-domain-notes/src/domain.rs:300`;
  `jig/tests/core/tests/suite/person_reads.rs` (a settings run with the
  asking family, its requester a person).

## Owns and may change

- **Owns:** the files in this spec's header. Siblings `party-reads`,
  `party-brief` and `withheld` run beside this task in other modules of
  `jig-domain-core`; touch none of theirs.
- **May change:** a call site's one line where `core-surface` wired
  `routing/party.rs`, if its arguments must grow, and say so.
- **Options, pre-approved:** if the notes child cannot tell a party's scope
  from its entry in `valid_change`, pass the scope in, as the survey found
  it must. If a run's task's authority holds the asking family through its
  charter's tools alone, read it as `authorize_tool` reads a family.

## Steps

1. The admission and the drop; the notes child's bounds.
2. The story and the rows; break the requester check once (admit any
   person's scope) to see its row fail, and restore.
3. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-domain-notes
   -p jig-core-world -p jig-notes-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-domain-notes -p jig-domain-core --lib`;
   `heavy cargo nextest run -p jig-core-world`; `heavy cargo nextest run -p
   jig-notes-world`. Commit and push.

## Report

As the implementer's text asks.
