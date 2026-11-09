+++
id = "withheld"
part = "party"
kind = "feature"
needs = ["core-surface"]
files = ["jig/crates/jig-domain-core/src/reads/withheld.rs", "jig/tests/core/tests/suite/withheld.rs", "jig/tests/client-protocol/tests/suite/turn_read.rs"]
effort = "high"
minutes = 180
+++

## Goal

Show a turn's private parts to the person who requested its task alone:
the core marks a turn withheld for every other reader and for every run's
read as its person, and a live call of a private tool carries its input to
the requester alone (`jig/docs/design/domain/engine.md`, 10.1, *Its runs'
parts are its party's*, and 7.3, *Answered*;
`jig/docs/design/client/domain/contract.md`, 6 and 7.3): fill
`reads/withheld.rs`, which `core-surface` made and wired.

Acceptance:

- A turn is carried withheld (`PageTurn::Withheld`,
  `EndedValue::WithheldTurn`) exactly when its task's requester is
  `Party::Person(p)` and the reader is any party but `p`, or when the read
  is a run's read as its person (`NamedAction::PersonRead`), whoever its
  person; otherwise as today. This holds for `Read::Turn`, for every
  activity page's turns, and for a page beside a line.
- A run's live document sends a call's input, made or being written
  (`call`, `writing`), of a tool named in `Settings.private_tools`, to a
  follower who is the task's requester alone; any other follower is sent
  the call with its tool and an empty input; the snapshot a watch opens
  with does the same (`documents/small.rs`, `run`).
- `jig/tests/core/tests/suite/withheld.rs`, one table story,
  `a_turn_is_withheld_from_all_but_its_person`, the core's settings naming
  one private tool: P's chat with a committed turn; P reads it carried; Q,
  a member of the project, reads it withheld, on its own and on an activity
  page; P's run reading it as its person reads it withheld; a task
  requested by a task reads carried to every reader. And one live row: Q
  following P's run sees a private call's tool with no input, P sees its
  input; another tool's input reaches both.
- jig's client protocol world (`tests/suite/turn_read.rs`), the clients'
  component given a `Private` with a title and a tool: a turn whose
  `system` text holds the title's section and which calls the tool, read by
  Q over the wire, decodes with the section's lines as
  `jig_protocol_transcript::WITHHELD` and the call's input `{}`; read by P,
  whole.
- Delete the **Not yet** entries `7.3, 10.1, *Its runs' parts are its
  party's*` in `jig/docs/design/domain/engine.status.md` and `6, 7.3: a turn
  and an activity page's turns read whole` in
  `jig/docs/design/client/domain/contract.status.md`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  sections 2 and 3.
- `jig/docs/design/domain/engine.md`, 10.1, *Its runs' parts are its
  party's*; 7.3, *Answered*; `jig/docs/design/client/domain/contract.md`,
  6, *A turn*, and 7.3, *The run's document*.
- `core-surface`'s report on the trunk and `reads/withheld.rs`' module doc:
  its functions and call sites.
- This cut's survey of the sites: `reads/rows.rs:650`, `:965`, `:1033`;
  `reads/activity.rs:9`; `reads/person.rs`; `routing/view/routes.rs:136`,
  `:214`; `documents/small.rs:204`, `:214`, `:221`;
  `jig-domain-views/src/text.rs:285`. A turn-backed `Reply` is read by its
  conversation's requester alone already (`reads/rows.rs:941`): nothing to
  withhold there.
- Patterns: `jig/tests/core/tests/suite/activity.rs:34`;
  `jig/crates/jig-domain-core/src/tests.rs:2758` and `:2783` (several
  watchers, a writing snapshot); `jig/tests/client-protocol/tests/suite/turn_read.rs:60`.

## Owns and may change

- **Owns:** the files in this spec's header. Siblings `party-runs`,
  `party-reads` and `party-brief` run beside this task in other modules of
  `jig-domain-core`; touch none of theirs.
- **May change:** a call site's one line where `core-surface` wired
  `reads/withheld.rs`, if its arguments must grow, and say so; the core
  world's fixture that sets the core's settings, for `private_tools`.
- **Options, pre-approved:** if a follower's party is not at hand where the
  live output is made, take it from the watch's binding as the survey found
  (`routes.rs:214`), and say so.

## Steps

1. The marks; the live filter.
2. The stories; break the mark once (carry Q's read) to see its row fail,
   and restore.
3. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-world -p
   jig-client-protocol-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p jig-domain-core --lib`; `heavy cargo nextest run -p
   jig-core-world`; `heavy cargo nextest run -p jig-client-protocol-world
   -p jig-engine-clients-world`; the readers of a turn's carrier, which this
   task changes in meaning: `heavy cargo nextest run -p temper-engine-smith
   --lib -E 'test(/see/)'` and `heavy cargo nextest run -p temper-client-world
   -E 'test(/live_text|call_line|chat/)'`. Commit and push.

## Report

As the implementer's text asks.

## Rebase (2026-11-09, step merge, foreman ui-ask-m4/withheld/foreman-1)

resolve the client-protocol manifest conflict by retaining all three added dev-dependencies
Rebase onto the landing trunk; the conflict is in `jig/tests/client-protocol/Cargo.toml` under `[dev-dependencies]`.
Trunk commit `da6bdaa446` (“Keep each person's preferences private to them”), merged by `91ef676e6d`, adds `jig-domain-notes.workspace = true`.
This branch's `95106f9e4d` adds `jig-protocol-transcript.workspace = true` and `smith-transcript.workspace = true`, both used by the new `turn_read.rs` story.
Resolve the adjacent insertions by keeping all three entries once, alongside the existing dependencies.
Preserve the automatically merged Cargo.lock and status-file changes.
