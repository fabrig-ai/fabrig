+++
id = "repository-tools"
part = "making-client"
kind = "feature"
needs = ["making-facts"]
files = ["temper/crates/temper-client-tools/src/making.rs", "temper/crates/temper-client-tools/src/table.rs", "temper/crates/temper-client-tools/src/root.rs", "temper/crates/temper-client-tools/src/forge.rs", "temper/crates/temper-client-tools/src/forges.rs", "temper/crates/temper-client-tools/src/jig/table.rs", "temper/crates/temper-client-tools/src/jig/argv.rs", "temper/crates/temper-client-tools/src/jig/render/outcome.rs", "temper/crates/temper-client-tools/src/tests/making.rs", "temper/crates/temper-client-tools/src/tests/jig/replays.rs", "temper/crates/temper-client-tools/src/tests/golden/**", "temper/docs/help/reference/ctl-repository.md", "temper/docs/help/reference/ctl-forge.md", "temper/docs/help/repository.md", "temper/docs/help/ctl.md", "temper/docs/help/index.md", "temper/docs/help/forgejo.md", "temper/crates/temper-docs/src/lib.rs"]
effort = "high"
minutes = 70
+++

# `temper ctl repository make` and `repository copy`

## Goal

Two tools send `MakeRepository`, each answering once the request is kept; `--until adopted` then follows the
row (`temper/docs/design/client/tools.md`, 5.3 and 5.4). `forge list` shows the rows.

Acceptance, in a new `src/tests/making.rs`, on `src/tests/forges.rs`'s scripted host:

- `make_sends_the_request_and_prints_kept`; `copy_lands_at_its_sources_owner_and_name`; `as_lands_there`.
- `an_address_refused_names_the_form_and_sends_nothing`, for `http://`, `git@` and more segments.
- `no_home_sends_home_false`, and `a_refused_home_names_no_home`, with the call to make under a new key.
- `until_adopted_waits_for_the_row_to_go`: status 0, printing `ORG/NAME is in PROJECT.`.
- `until_adopted_stops_on_a_failed_row`: status 1, the row's words.
- `until_adopted_past_timeout_is_status_2`, printing the row's state.
- `timeout_without_until_is_refused`.
- `the_same_key_sent_again_answers_kept_and_waits_again`.
- `forge_list_shows_the_rows`, its text and `--json`.
- Help: `help_repository_make.txt`, `help_repository_copy.txt` and their registration in `tests/help.rs`;
  `help_groups.txt`, the group's line.
- `heavy cargo nextest run -p temper-client-tools -p temper-docs -p temper` pass.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, T3 and T4.
- `temper/docs/design/client/tools.md`, 5.3 and 5.4.
- `temper/docs/design/draft/repository-copies.md`, 5.3, 5.4 and 10.

## Owns and may change

- The files of the header. **A departure from the implementer's text:** `temper/docs/help/` is yours here.
  `ctl-repository.md` regenerates with `SKEIN_CODEGEN_WRITE=1` (`temper/crates/temper/src/tests/commands.rs`,
  274); the pages named in the header by hand, one or two lines each, in their voice. They are read by
  tests (`.config/gate/read-by-tests.txt`).
- **The rows:** new rows in a new `src/making.rs`, on `src/adopt.rs`'s `ROW`, appended at the **tail** of
  `TOOLS` (`table.rs`, 109): inserting beside `adopt` shifts `KEPT_INDICES` (`tests/words.rs`), the `*_AT`
  constants (`jig/rows.rs`, 582) and the literals of `tests/help.rs`.
- **Tests that read the list, each its new entries alone:** `tests/words.rs` (`KEPT_INDICES`, placeholders),
  `tests/help.rs` (`USAGES`), `tests/fixtures.rs` (`EVERY_TOOL`, `TOOL_FORMS` 75 to 77), `tests/table.rs`
  (`KEY_REQUIRED`), `tests/printed.rs`, `temper/crates/temper/src/tests/commands.rs`.
- **Placeholders:** `ORG` and `REPOSITORY` exist (`jig/table.rs`, 207); `ADDRESS` and `ORG/NAME` are new, at
  `jig/table.rs`, 169. Stay within `jig/`'s tables: no new mechanism in `jig/wait.rs`.
- **`--until adopted`:** the act's answer first, then the `forges` watch, as `forge list --until` is judged
  (`forges::holds`, `forges.rs`, 767). Adopted is the row gone with its forge still listed. An act taking
  `--until` is new: the two tools own the check that `--timeout` needs it, since `check_words`
  (`jig/argv.rs`, 365) guards only rows with a `since`.
- **The send:** `forge::send`'s pattern (`forge.rs`, 73), the named key from `crate::key::engine`, the
  restart's `restored` (217), and `Told::Answered`.
- **The project** must exist: `--project` on an unknown one is "unknown project", as `repository adopt`'s.
- Remove "5.3: ..." from `temper/docs/design/client/tools.status.md`.
- Pre-approved:
  - `forges.rs`'s render (802, 917, 972): a line per row, under the forge's, by `making::phrase`.
  - `forge list --until settled` waits for a row's end too: a `Failed` row is settled.
  - `--no-home` in `make` and `copy` is `adopt`'s flag, in its words.

## Steps

1. The rows and their tests' lists. 2. The send. 3. The wait. 4. `forge list`. 5. Help. 6. The tests, each
biting: treat row-gone as still waiting, and read a failed row as adopted, each once. 7. The implementer's
checks; commit and push.

## Report

As the implementer's text asks; how an act's `--until` is held in the table, in a line a later tool could
copy.

## Answers

- 2026-11-04. May this task add the two `repository make` and `repository copy` cases to `DIRECT_CALLS` in
  `temper/crates/temper-client-tools/src/tests/jig/replays.rs`? Yes: the list is one of the "tests that read
  the list", which the spec missed, and the task's suite cannot pass without it. The file is now in the
  header. Add those two entries alone, on the same terms: `direct` replays each in text and JSON, comparing
  the request, the key and the answer.
- 2026-11-04. May this task change the `ctl-repository` summary in `temper/crates/temper-docs/src/lib.rs`, or
  must the old group line stay? Change it: the file is now in the header, for that one literal alone. The
  group's line is the new one, `Repositories: make, copy or adopt one into a project`, in `table.rs`, its
  goldens and the generated page; the tools now make and copy, so the old line would be false. `summary` at
  `lib.rs`, 400, becomes `Repositories: make, copy or adopt one into a project.`, and nothing else there
  moves. Then run what the last run left unobserved: the failing docs test, the clippy, and the acceptance's
  whole `heavy cargo nextest run`.

## Rebase (2026-11-04, step merge, foreman repository-copies-1/repository-tools/foreman-3)

combine forge resource-host help with repository-making help in the three conflicting paths
The merge failed on overlapping help text, introduced on main by `82618af6f5` (“Expose forge resource hosts so charter documents can name their forge”).
Resolve these paths together:
- `temper/crates/temper-client-tools/src/forges.rs`, `READ.prints`
- `temper/crates/temper-client-tools/src/tests/golden/help_forge_list.txt`
- `temper/docs/help/reference/ctl-forge.md`
Use this combined wording in all three, retaining `Prints: ` in the golden and reference:
“A line per forge, `forge N`, its resource host, kind, mode and connection, its App, installations and repositories being made, then `digest: forges`”.
In `forges.rs`, preserve main’s `encode_identity` and `text_identity` helpers and their calls, including the JSON `host` field.
Preserve the branch’s making-row rendering, `settled` check, `ROWS_JSON_DEPTH = 10`, and scoped `too_many_lines` expectation.
Retain the automatically merged host assertions and repository make/copy registrations.
Verify the combined help with `tests::help::the_help_prints_as_its_golden_files` and `tests::help::the_forge_reference_is_what_the_table_renders`; verify rows with `tests::making::forge_list_shows_the_rows`, through heavy.
Then complete the spec’s clippy and whole `heavy cargo nextest run -p temper-client-tools -p temper-docs -p temper` checks.
