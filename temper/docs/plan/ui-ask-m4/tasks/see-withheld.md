+++
id = "see-withheld"
part = "route"
kind = "feature"
needs = ["core-surface"]
files = ["temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/see/person/runs.rs", "temper/crates/temper-engine-smith/src/tests/see_person.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/tests/**"]
effort = "high"
minutes = 120
+++

## Goal

Name temper's private parts to jig and withhold them from `see`: the
brief's "Kept about this person" and `remember`'s calls, given to the
core's and the clients' settings at temper's start, and every turn `see`
renders from a withheld carrier projected first
(`temper/docs/design/domain/agent.md`, 8.4.4, *Seen by the person alone*;
`jig/docs/design/domain/engine.md`, 10.1, *Its runs' parts are its
party's*).

Acceptance:

- `temper_engine_smith::PRIVATE_TOOLS` and `temper_engine_smith::private()`
  exist as `temper/docs/plan/ui-ask-m4/notes/contracts.md`, section 5 gives
  them, beside `PREFERENCES_TITLE`, with doc comments citing 8.4.4.
- temper-service gives them at its start: the core's
  `Settings.private_tools` holds `remember` (`system.rs`, where the core's
  configuration is made, `jig_protocol_config::core(..)`), and the clients'
  component's `Settings.private` is `private()` (`clients.rs`,
  `Clients::new`'s `kit::Settings`); a row of an existing temper-service
  start test holds both.
- `see/person/runs.rs`' `decode` (and any other `see` renderer of turns)
  takes a withheld carrier (`EndedValue::WithheldTurn`,
  `PageTurn::Withheld`) through `jig_protocol_transcript::withhold` with
  `private()` before it decodes, and a carried one as today, replacing
  `core-surface`'s arms; an undecodable projection is rendered as an
  undecodable turn is.
- A row of `src/tests/see_person.rs`'s turn table
  (`person_answers_read_as_named_bounded_lines`, or its activity sibling):
  a withheld turn whose brief holds "## Kept about this person" with two
  lines and whose turn calls `remember` renders the heading, the withheld
  line and `remember`'s call without its input; the same turn carried
  renders whole.
- Delete `temper/docs/design/domain/agent.status.md`'s **Not yet** entry
  `8.4.4, *Seen by the person alone*`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  sections 2, 3 and 5.
- `temper/docs/design/domain/agent.md`, 8.4.4, *In the brief* and *Seen by
  the person alone*; 8.4.1's `task N turn A.T` and `task N activity` rows.
- `jig/docs/design/domain/engine.md`, 10.1, *Its runs' parts are its
  party's*; 7.3, *Answered*.
- `temper-engine-smith/src/see/person/runs.rs:15` (`decode`), `:187` and
  `:273` (a turn's rendering); `temper-service/src/system.rs:242`;
  `temper-service/src/clients.rs:230`.

## Owns and may change

- **Owns:** the files in this spec's header.
- **Options, pre-approved:** if the core's settings are made in another
  place than `system.rs` by the time this runs, set them where the core's
  configuration is made, and say so; if no start test fits the row, add
  one beside the nearest that reads the core's settings.

## Steps

1. The names; their settings at the start; the projection in `see`.
2. The rows; break one rule (render a withheld carrier whole) to see its
   row fail, and restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-smith -p temper-service
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-smith --lib`; `heavy cargo nextest run -p temper-service
   --lib`. Commit and push.

## Report

As the implementer's text asks.
