+++
id = "pg-ctl"
part = "playground"
kind = "feature"
needs = ["pg-playground-pages", "pg-forge-remove"]
files = [
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/demo.rs",
  "temper/crates/temper-client-tools/src/table.rs",
  "temper/crates/temper-client-tools/src/tests/forges.rs",
  "temper/crates/temper-client-tools/src/tests/playground_reset.rs",
]
effort = "high"
minutes = 45
+++

## Goal

`temper ctl` gains the run's two removals, so scripts and the process
stories (`bx-stories`) drive them as a person's pages do: `playground
reset`, which sends `EndPlayground` and returns once the reset answers
(draft `temper/docs/design/draft/playground-reset.md`, 6.2), and `forge
remove FORGE`, which sends `RemoveForge` (draft 4). `forge setup`'s
forge number becomes optional and unused, since `SetUp` names no forge
(contracts, 5).

Departure from the implementer's text, stated: the ctl help pages are
generated from the tool table and pinned by the `temper` crate's
`every_ctl_group_page_is_what_the_table_renders`
(`temper/crates/temper/src/tests/commands.rs`); `docs/help/` is not
yours, so that test fails on this branch until `pg-ctl-help` writes the
pages. Run it once, put its "the page should be:" output for each
changed group in the report, list it under `## Left`, and report
`Done:` with it red; every other test passes.

Acceptance:

- `playground reset [--key KEY]`: sends `w::Request::EndPlayground`
  (through `temper_client_domain::forge::demo::end_ask`); prints
  `reset` (exit 0) once answered `reset`; `refused: not offered`
  (exit 2) for `end_refused`; `unknown` (exit 1) otherwise; `--json`
  the record `playground-reset` with the outcome as `demo-start`'s
  JSON writes its own; a kept answer prefixed `kept: playground-reset `
  as `demo-start`'s is;
- `forge remove FORGE [--key KEY]`: sends `w::Request::RemoveForge {
  forge }` (through `temper_client_domain::forge::remove::ask`); prints
  `removing` (exit 0); `refused: not offered`, `refused: no such
  forge`, `refused: not an owner` (exit 2); `unknown` (exit 1);
  `--json` the record `forge-remove`;
- `forge setup [FORGE] ...`: the positional is optional and its value
  unused (the engine sets up the box's owned Forgejo); every other flag
  as today;
- each new tool is in `table::TOOLS` (the playground group's after
  `playground start`, the forge group's after `forge setup`), the
  playground group's line "The playground: start it, or reset it";
- the tests below pass; the tool table's other tests (usage lines,
  words) updated for the new tools.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 5).
- `temper/docs/design/draft/playground-reset.md`, 4 and 6.2.
- `temper/docs/design/client/tools.md`, 5.4 and 5.5 (area 6, the forge
  tools; how a tool is declared and printed).
- `temper/docs/plan/playground-reset-r4/tasks/pg-playground-pages.md`
  (`end_ask`, `end_answer`) and `tasks/pg-forge-remove.md`
  (`remove::ask`, `remove::answer`).
- The pattern: `DEMO_START` and its print
  (`temper-client-tools/src/forges.rs`, `src/demo.rs`) and its tests
  (`src/tests/forges.rs`, `demo_start_answers_print_their_words_json_and_status`,
  `demo_start_keeps_drawn_keys_and_recovers_its_own_answer`).

## Owns and may change

Owned: the `files` above (`tests/playground_reset.rs` is new).

Neighbour files, edits allowed:

- `temper-client-tools/src/tests.rs`: one `mod` line;
  `src/tests/help.rs`, `src/tests/words.rs`: their expectations for the
  new tools and the changed `forge setup` usage line.
- `temper/crates/temper/src/tests/commands.rs`: nothing (its test stays
  red until `pg-ctl-help`, as stated).

Pre-approved:

- the two tools share `forges.rs`' pending table (`state.pending`) with
  an intent kind for each (`SettingUp`'s `intent` becomes an enum, or a
  parallel table); default: a small enum `Pending::{SetUp, Start,
  Reset, Remove}` if the table's value needs it;
- if the tool table bounds tools per group or in total, raise the bound
  the check names and say so.

## Steps

1. **Tools** (`forges.rs`): `pub const PLAYGROUND_RESET: Tool` (name
   `playground reset`, line "Reset the playground: its project, forge
   and account are removed, and the welcome opens at its Forge step",
   prints "`reset`", result `playground-reset`, `Keyed::Optional`, area
   6, notes saying it returns once the playground is gone and that
   `refused: not offered` (exit 2) means no playground runs) and `pub
   const REMOVE: Tool` (name `forge remove`, a required positional
   FORGE 1..=65535, line "Remove a forge whole: its live work cancelled,
   its records erased, its box group removed", prints "`removing`",
   result `forge-remove`, `Keyed::Optional`, area 6, notes naming what
   stays on the forge as the Forges page does: a managed Forgejo's bot
   and hooks, a GitHub App). `owns` gains both; `send` routes them as
   `DEMO_START` is routed; `told` prints them.
2. **Print** (`demo.rs`): `pub(crate) fn print_reset(outcome, as_json,
   kept, limits) -> jig::Printed` and `pub(crate) fn print_remove(...)`,
   beside `print`, with the words and statuses of the acceptance.
3. **Table** (`table.rs`): the tools and the group's line.
4. **Tests** (`temper-client-tools`' unit tests):
   - `tests/playground_reset.rs`:
     `playground_reset_sends_end_playground_and_prints_reset`
     (`["playground", "reset", "--key", "reset"]`: one `Send` whose
     ask decodes to `w::Request::EndPlayground`; answered `reset`:
     stdout `reset`, exit 0; `--json`:
     `{"operation":"playground-reset","outcome":{"reset":{}}}` or the
     shape `demo-start`'s JSON gives);
     `playground_reset_refused_is_exit_two`;
     `forge_remove_sends_remove_forge_and_prints_removing` (`["forge",
     "remove", "3"]`: `w::Request::RemoveForge { forge: 3 }`; `removing`,
     exit 0);
     `forge_remove_refusals_say_why` (each refusal, exit 2);
     `forge_setup_without_a_forge_sends_set_up`.
   - Narrow runs: `heavy cargo nextest run -p temper-client-tools --lib
     -E 'test(playground_reset::) | test(forges::) | test(help::) |
     test(words::)'`.
5. Checks, once: `cargo fmt`; `heavy cargo clippy -p temper-client-tools
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-tools`; `heavy cargo check -p temper-client-shell-cli
   --tests`, `-p temper --tests`; then once `heavy cargo nextest run -p
   temper --lib -E 'test(/every_ctl_group_page_is_what_the_table_renders/)'`
   (a unit test of the binary's crate, `src/tests/commands.rs`; check
   the count is 1) to capture the pages for the report. Commit, push.

## Report

fab's sections; `## Interfaces`: the two tools (name, forms, result),
`print_reset`, `print_remove`; `## Left`: the help test, with the
rendered pages for `ctl-playground.md` and `ctl-forge.md` verbatim.
