+++
id = "limits-jig"
part = "pages"
kind = "interface"
needs = []
files = ["jig/crates/jig-protocol-config/src/names.rs", "jig/crates/jig-protocol-config/src/lib.rs", "jig/crates/jig-protocol-config/src/overrides.rs", "jig/crates/jig-protocol-config/src/worker.rs", "jig/crates/jig-protocol-config/src/tests/names.rs", "jig/crates/jig-protocol-config/src/tests/overrides.rs", "jig/crates/jig-protocol-config/src/tests.rs"]
effort = "high"
minutes = 120
+++

# limits-jig: jig's limits by name, beside its parser

## Goal

jig gives every limit its override parsers take as a row of a names table, with its unit, a
line and its value's reader (contract C3), so that temper renders them as the reference page
`limits-engine` (`limits-pages`). The parser looks a name up in the table first, so a name it
takes is a row; a test applies every row, so a row is a name it takes.

Acceptance:

- `jig_protocol_config::names` with C3's `Unit`, `Name`, `NAMES` (and `WORKER_NAMES` where a
  worker's limits are their own type) and `find`, its module doc citing
  `jig/docs/design/protocol/configuration.md`, 6 and `temper/docs/design/protocol/documentation.md`, 8.
- `overrides::apply` (and a worker's parser, `worker.rs`, if it has its own) refuses a name
  `find` lacks before its match, with the error an unknown name gets today; the match is
  otherwise unchanged.
- Every line is one sentence, at most 120 bytes, saying what the limit bounds in the words of
  the field's own doc comment or jig's `protocol/configuration.md`, 6; generic to jig's
  applications (it names no temper term).
- Step tests in `src/tests/names.rs`: `every_row_is_a_name_the_parser_takes` (each row's
  name applied at its small value succeeds and leaves `value` reading that value);
  `every_name_the_parser_took_is_a_row` (the test-only `SMALL` list in
  `src/tests/overrides.rs` and every row agree, the `SMALL` list then read from `NAMES`, its
  198 pairs no longer typed in); `each_line_is_within_its_bound`; `an_unknown_name_is_refused`.
- The focused tests of `jig-protocol-config` and its readers' (`heavy cargo check --workspace
  --all-targets` clean) pass.

## Inputs

- `jig/docs/design/protocol/configuration.md`, 6 (named limits).
- `temper/docs/design/protocol/documentation.md`, 8 (`limits` and `limits-engine`).
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C3.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`; in `lib.rs` the `pub mod names;` line, in
  `tests.rs` its `mod names;`.
- This crate is also changed by `public-reading-build` on its trunk (its `public-limits` task
  added limits to `jig-protocol-config`): keep `overrides.rs`' hunk to the lookup's lines, so
  that a catch-up meets one small hunk there; the names it adds reach this table by a later
  repair when that plan lands, not by you.
- Options, pre-approved:
  - Where a name's field has no doc comment and jig's design gives it no words, write the
    line from what the parser's arm sets, and list each such name in the report.
  - Where an override sets several fields (`accounts.token`), `value` reads the first the
    arm sets.

## Steps

1. `names.rs`: the types, then the rows in the parser's order, owner by owner.
2. `apply`'s lookup; the tests.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-config --all-targets -- -D warnings`; `heavy cargo nextest run -p
   jig-protocol-config`; the workspace check.
4. Commit ("jig-protocol-config: every override's name a row, beside its parser
   (documentation.md, 8)") and push.

## Report

As the implementer's text says, and the table's length by owner.
