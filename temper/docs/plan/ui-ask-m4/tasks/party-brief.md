+++
id = "party-brief"
part = "party"
kind = "feature"
needs = ["core-surface"]
files = ["jig/crates/jig-domain-core/src/routing/run/preferences.rs", "jig/tests/core/tests/suite/party_brief.rs"]
effort = "high"
minutes = 160
+++

## Goal

Put a person's preferences, whole, in the brief of each run that explains
to them: the core section `preferences` (`jig/docs/design/domain/engine.md`,
9 and 10.1, *Read by those runs, whole in their brief*); fill
`routing/run/preferences.rs`, which `core-surface` made and wired.

Acceptance:

- Where a run's task's requester is `Party::Person(party)` and its
  authority holds the asking family, its brief's preparation gathers
  `Scope::Party { party }`'s index, at most `notes.party_entries` lines,
  through the notes child's `Index` as the notes index is gathered; plans
  the section `preferences`, required, its budget
  `CoreBriefBudgets.preferences`; and writes its text as
  `temper/docs/plan/ui-ask-m4/notes/contracts.md`, section 2 says: each
  line `{place}. {description}`, in name order, or `None kept.`. Any other
  run's brief has no such section, and the notes index never lists a
  party's scope.
- A gather that fails counts as the notes index's do (9.1): no room,
  capacity, `notes: no room for the index`; refused, permanent, `notes: the
  index could not be read`, each with the section's name `preferences` in
  place of `notes` where 9.1's table names the section.
- `jig/tests/core/tests/suite/party_brief.rs`, its lines seeded as store
  rows before the start (`jig/tests/core/tests/support/reads.rs:193`), one
  table story, `an_asking_run_reads_its_persons_preferences_whole`, each row
  the brief's `preferences` section as the root receives it: P's asking run
  with two lines; with none, `None kept.`; with twelve lines of 120 bytes,
  whole; Q's asking run, Q's lines alone; P's run without the asking
  family, no section; P's notes index listing no party line.
- temper's stock briefs still fit: `heavy cargo nextest run -p
  temper-service --lib -E 'test(/stock_brief|ask_bound/)'` passes, the first
  request with `None kept.` within 16,384 bytes.
- Delete `jig/docs/design/domain/engine.status.md`'s **Not yet** entry `9,
  10.1, *Read by those runs*`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  section 2.
- `jig/docs/design/domain/engine.md`, section 9 and 9.1; 10.1, *Read by
  those runs, whole in their brief*.
- `core-surface`'s report on the trunk and `routing/run/preferences.rs`'
  module doc: the stub's functions and call sites.
- Patterns: `routing/run/preparation.rs:345` and `:365` (the notes index's
  gather), `:530` (`plan_note_index`); `runs/brief_text.rs:31`
  (`note_index_text`); `jig/crates/jig-domain-core/src/tests/runs/notes_room.rs`;
  `jig/tests/core/tests/stories/brief_failure.rs:113`.

## Owns and may change

- **Owns:** the files in this spec's header. Siblings `party-runs`,
  `party-reads` and `withheld` run beside this task in other modules of
  `jig-domain-core`; touch none of theirs.
- **May change:** a call site's one line where `core-surface` wired
  `routing/run/preferences.rs`, if its arguments must grow, and say so;
  a row in `jig/crates/jig-domain-core/src/tests/runs/notes_room.rs` for a
  gather refused.
- **Options, pre-approved:** if the brief planner can cut a required
  section within its budget, check the section's room against its whole
  text before planning and fail the brief as a required section that
  cannot fit does (9.1), and say so.

## Steps

1. The gather, the plan and the text.
2. The story; break the order once (reverse the lines) to see its row
   fail, and restore.
3. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   jig-domain-core --lib`; `heavy cargo nextest run -p jig-core-world`; the
   temper-service run above. Commit and push.

## Report

As the implementer's text asks; give the first request's parts as the
temper-service test prints them.

## Answers

2026-11-09. *May party-brief change the notes child's `Index` read so a
party's scope is bounded by `notes.party_entries` rather than `notes.lines`?*
Yes; the change is this task's, since the twelve-line row cannot hold without
it, and engine.md 10.1 says the section holds the scope's lines, every one.
In `jig/crates/jig-domain-notes/src/read.rs`, `drive`'s `Kind::Index` arm
bounds an index whose scopes are exactly one `Scope::Party { .. }` by
`most.min(limits.party_entries)`, and every other index by
`most.min(limits.lines)` as now; every `match` on `Scope` lists every
variant. Also edit the `lines` doc line in
`jig/crates/jig-domain-notes/src/limits.rs` to say a party's scope answers up
to `party_entries`. Don't raise any profile's `notes.lines` or change a
fixture's bound to get round it. Don't touch `jig-domain-notes/src/tests.rs`,
`domain.rs` or `boundary.rs` (`party-runs` owns them): the story's
twelve-line row in the small profile is the check, and `heavy cargo nextest
run -p jig-domain-notes --lib` must still pass. Name both edits in the report.
