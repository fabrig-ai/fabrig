+++
id = "configuration-pages"
part = "pages"
kind = "feature"
needs = ["members-jig", "limits-pages", "help-stock"]
files = ["temper/crates/temper-service/src/startup/members.rs", "temper/crates/temper-service/src/startup/reference.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/application.rs", "temper/crates/temper-service/src/startup/application/**", "temper/crates/temper-service/src/tests/members.rs", "temper/crates/temper-forge-setup/src/machine.rs", "temper/crates/temper/src/tests/configuration.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper-docs/src/lib.rs", "temper/docs/help/reference/configuration.md", "temper/docs/help/reference/configuration-engine.md", "temper/docs/help/*.md", "temper/docs/design/protocol/documentation.status.md"]
effort = "high"
minutes = 180
+++

# configuration-pages: the reference pages `configuration` and `configuration-engine`

## Goal

temper gives every member of the document's top level and `application`'s shapes a row of a
members table beside its decoders (contract C4), as `members-jig` gave jig's, and renders the
two reference pages: `configuration` (temper's) and `configuration-engine` (jig's), each
row's "init writes" read from `temper init`'s document, checked in under
`docs/help/reference/` and held equal to their rendering (temper's
`protocol/documentation.md`, 8 and 6). Every member is read at a start, so each page says once
that a change to any member needs a restart, and the concept pages' `## Limits` sections link
`temper help configuration`. This task's departure from the implementer's text: it checks in
the two rendered pages, their `PAGES` rows, and one link line in the written concept pages.

Acceptance:

- `temper_service::startup::members` (C4's types and functions, temper's rows) for the top
  level (`engine`, `workers`, `application`) and `application`'s shapes (`machine`, `demo`,
  `limits`, `workers`, `clients`, `sign_in`, `landing`, `webhooks` and their nested shapes),
  `temper-forge-setup`'s `machine` decoder reading its members through it; the gone members
  (`application.charter`, `application.landing`'s rules) rows saying where they went, as their
  refusals say (`CharterStored` and its kin).
- temper's decoders (`startup.rs`' shape and members, `startup/application.rs` and
  `application/`) read their members through the table as C4 says jig's do.
- `configuration_reference` and `engine_configuration_reference` (in
  `startup/reference.rs`, re-exported where `limits`' renderers are) render C4's pages, each
  within 32,768 bytes, each saying once that a member changes only at a restart.
- The two pages checked in with their heads, and their `PAGES` rows in the table's order
  (`configuration` and `configuration-engine` after the `ctl-*` block, before `host-tools`).
- Step tests: in the `temper` crate, `tests/configuration.rs`,
  `the_pages_configuration_are_what_the_tables_render` (with `SKEIN_CODEGEN_WRITE=1`
  rewriting them) and `every_member_init_writes_is_a_row` (every member path of `temper
  init`'s filled document, `engine`'s and `application`'s, is a row of jig's or temper's
  table); in `temper-service`, `every_row_is_a_member_its_decoder_takes`.
- Each written concept page under `temper/docs/help/` that has a `## Limits` section gains one
  line in it naming `temper help configuration` (what a restart is needed for), its other text
  as it is; `temper-docs`' link and template tests pass.
- temper's `docs/design/protocol/documentation.status.md`: the sentence "8: the reference
  pages `configuration` and `configuration-engine`." removed.
- The focused tests of `temper-service`, `temper-forge-setup`, `temper-docs`, `temper-init`
  and `temper` pass.

## Inputs

- `temper/docs/design/protocol/documentation.md`, 2.1, 6 and 8.
- `temper/docs/design/protocol/configuration.md`, sections 2 to 6 (the document's members)
  and 13.5 (the members gone).
- `temper/docs/design/protocol/setup.md`, 8.3 (what `temper init` writes).
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C4.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`; in `temper/crates/temper/src/tests.rs` the `mod
  configuration;` line; in `startup.rs`, its readers' shape names alone (`room-checks` changes
  it beside this task's: keep the hunks apart).
- Options, pre-approved:
  - If a written concept page's `## Limits` section would pass its kind's bound with the line,
    shorten the line, never the page's other text.
  - If `temper init`'s document can only be filled through `temper-init`'s own function, call
    it from the `temper` crate's test, which already depends on `temper-init`.
  - A member with no design words and no doc comment: its line from what its decoder sets,
    listed.

## Steps

1. temper's table, the decoders' reading through it, `machine`'s.
2. The renderers; the tests; write the pages with `SKEIN_CODEGEN_WRITE=1`, then check them
   clean.
3. The `PAGES` rows; the concept pages' line; the status sentence.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-service -p temper-forge-setup -p temper-docs -p temper --all-targets -- -D
   warnings`; the focused tests above; the citations check.
5. Commit ("temper: the reference pages configuration and configuration-engine
   (documentation.md, 8)") and push.

## Report

As the implementer's text says, and each page's bytes.
