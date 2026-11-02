+++
id = "panel-steps"
part = "change-panel"
kind = "feature"
needs = ["tiers-verification"]
files = ["temper/crates/temper-engine-domain-forge/src/panel.rs", "temper/crates/temper-engine-domain-forge/src/tests/panel.rs", "temper/crates/temper-client-protocol-wire/**", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-client-domain/**", "temper/crates/temper-client-view/**", "temper/crates/temper-client-tools/src/change.rs", "temper/crates/temper-client-tools/src/tests/**", "temper/tests/client/**", "temper/docs/help/reference/ctl-change.md"]
effort = "high"
minutes = 240
+++

# The change panel after landing: verifying, red, cleared; `--until cleared`

## Goal

A change's panel, as `change show` and the change's page read it, goes on past `landed`:
`verifying` with the tiers that have not judged it, `red` with the tiers that blame it, and
`cleared` (or landed and uncleared); `change show TASK --until cleared` waits for it. The walk
reads a verdict this way.

Acceptance:

- `temper/docs/design/client/forge.md`, 2.1's rows for `verifying`, `red` and `cleared`, and
  `temper/docs/design/client/tools.md`, 5.4 (`landed` met by every step from `landed` on;
  `cleared` met at `cleared` or landed and uncleared) and 5.5 (`change show`'s following call:
  `--until landed` before it lands, `--until cleared` while verifying or red).
- The forge's panel (`panel.rs`) gives the steps from the change's state (`Landed` with a
  `Verification` row is `verifying`; `Red`; `Cleared` and `Uncleared`) and the tiers' rows: for
  each tier following its branch, its name and its verdict or its run in flight; for a red one,
  the run and the failing tests blamed, counted, or many. A `Landed` change with no row (no tier
  follows its branch) stays `landed`.
- The client wire (`schema/client-wire-v1.schema`) appends `verifying`, `red` and `cleared` to
  `Step` after its last variant, each with a record carrying the merge commit and the tiers'
  lines; generated code and goldens regenerated in place; the service's translation
  (`temper-service/src/family.rs`) fills them.
- `temper-client-tools`' `change show`: the `--until` set gains `cleared`; `landed` is met from
  `landed` on; the text prints the step's line as client/forge.md 2.1 words it (`step:
  verifying · TIER run N running`, `step: red · TIER run N · 2 tests`, `step: cleared · TIER run
  N`, `step: landed, uncleared`); the next call as tools.md 5.5 says; the JSON carries the same.
  Its notes text names `--until cleared`. `temper/docs/help/reference/ctl-change.md` is
  regenerated from the table by its test's write mode (`SKEIN_CODEGEN_WRITE=1`, test
  `every_ctl_group_page_is_what_the_table_renders` in package `temper`): this task's one
  `docs/help` file, a generated one.
- The client domain and view show the new steps where they show `landed` today
  (`temper-client-domain/src/forge/change.rs`, `temper-client-view`'s change page and delegation
  card), in the panel's words.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
  'test(/panel|views/)'`; `-p temper-client-protocol-wire`; `-p temper-service --lib -E
  'test(/family/)'`; `-p temper-client-tools --lib`; `-p temper-client-domain`;
  `-p temper-client-view`; `-p temper-client-world --test suite -E 'test(/change/)'`; `-p temper --lib -E
  'test(=tests::commands::every_ctl_group_page_is_what_the_table_renders)'`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 2 and 4.
- `temper/docs/design/client/forge.md`, 2.1 and 2.5; `temper/docs/design/client/tools.md`, 5.4
  and the `change show` row of 5.5.
- Readers of today's `landed` step, by a survey: `temper-engine-domain-forge/src/panel.rs` (about
  34, 95, 137) and its tests `src/tests/{panel,views}.rs`; `temper-service/src/family.rs` and
  `src/tests/family.rs`; `temper-client-protocol-wire/src/tests/fixtures.rs`;
  `temper-client-tools/src/change.rs` (the flag about 35, notes about 40, conditions about 745
  and 754, the next call about 1148) and `src/tests/{change,landing}.rs`;
  `temper-client-view/src/tests/change.rs`, `tests/suite/change_page.rs`;
  `temper-client-domain/src/forge/change.rs`, `src/tests/{change,panel}.rs`,
  `tests/suite/change.rs`; `temper/tests/engine/domain/src/oracle/durable.rs`;
  `temper/tests/client/tests/suite/{change_panel,change_page,change_panel_edges}.rs`.

## Owns and may change

- Owns the files in the header for this wave, and the readers listed above for the new arms and
  goldens.
- Not this task's: the forge's `tiers.rs` and `verification.rs` (read their rows through the
  forge's own accessors; add a read-only accessor in `panel.rs` if one lacks), the concept help
  pages (`help-pages`).

Pre-approved options:

- If the client wire's `Step` cannot take a record per new variant within its bounds, carry the
  tiers' lines in one record shared by the three, and say so.
- If the web view has no place for the tiers' lines, show the step's name and its first tier's
  line, and list the rest as the report's `## Left`.

## Steps

1. The forge's panel steps and their unit tests.
2. The wire, the service's translation, regeneration.
3. `ctl`'s conditions, text, JSON and the regenerated reference page; then the client domain and
   view.
4. `cargo fmt`; clippy over every package you changed; the tests in Goal, counts reported.
5. Commit in steps that say why; push your branch.

## Report

As the implementer's text says, and also: the panel's lines as printed for each step; the
regenerated files.
