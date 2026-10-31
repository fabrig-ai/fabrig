+++
id = "cl-tools"
part = "client"
kind = "feature"
needs = ["cl-connect", "cl-remove", "cl-play-domain"]
files = [
  "temper/crates/temper-client-tools/**",
  "temper/crates/temper-client-shell-cli/**",
  "temper/docs/help/reference/ctl-forge.md",
  "temper/docs/help/reference/ctl-playground.md",
]
effort = "high"
minutes = 75
+++

## Goal

`temper ctl` on contract 5: `forge setup` names no forge and prints the
number its answer names; `forge connect` makes a new GitHub with no
number and goes on with a kept one given one (draft 6.4); `forge remove
FORGE` removes a forge (draft 4); `playground end` resets the playground
and returns once there is none (draft 6.2); `playground start` sends
`StartPlayground`. D4's process tier and D2's runners call these.

Acceptance (`temper-client-tools`, `src/tests/`):

- `forges.rs`: `forge_setup_names_no_forge_and_prints_the_answered_number`;
  `forge_setup_with_a_number_is_a_usage_error`;
  `forge_remove_sends_the_named_removal_and_prints_removing`;
  `forge_remove_refused_not_offered_exits_two`;
  `forge_remove_of_an_unknown_forge_exits_two`.
- `forge_connection.rs`: `forge_connect_github_without_a_number_makes_one`;
  `forge_connect_github_with_a_kept_number_goes_on`.
- new `playground.rs` (or the existing demo tests' file):
  `playground_end_prints_resetting_then_ended_once_no_playground_reads`;
  `playground_end_refused_not_offered_exits_two`;
  `playground_start_sends_start_playground`.
- `help.rs`: the help goldens and `the_help_is_the_page_ctl_then_the_tools`
  pass, regenerated; `temper`'s
  `tests::commands::every_ctl_group_page_is_what_the_table_renders`
  passes with the regenerated reference pages.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 5.
- `temper/docs/plan/playground-reset-r2/tasks/cl-wire.md`,
  `cl-remove.md` (`remove_key`), `cl-play-domain.md` (`end_key`,
  `READ_AGAIN`, the read), `cl-connect.md` (the set up's and connect's
  domain items).
- `temper/docs/design/draft/playground-reset.md`, 4, 6.2, 6.4.
- `temper/docs/design/client/tools.md`, the forge and playground tools
  (`rg -n 'forge setup|playground' temper/docs/design/client/tools.md`).

## Owns and may change

Owns the files in the header.

**Departure (stated):** `temper/docs/help/reference/ctl-forge.md` and
`ctl-playground.md` are generated from the tools table by `temper`'s
`every_ctl_group_page_is_what_the_table_renders`; regenerate them with
`SKEIN_CODEGEN_WRITE=1` on that test, never by hand. The hand-written
help pages (`temper/docs/help/{forge,playground}.md` and their kin) are
the lead's: name in the report each sentence there the new tools make
wrong.

Forced edits (scope.md): every caller of `forge setup N` or `forge
connect N github…` the change breaks drops its number (the argv only):
`temper/tests/client/tests/{suite/forges.rs,qa_all/qa_first_repository.rs}`,
`temper/tests/client-browser/src/lib.rs`,
`temper/tests/real-loop/{src/github.rs,tests/suite/github.rs}`,
`temper/crates/temper/tests/{process.rs,qa_all/*.rs}`, and any other `rg
-n '"forge", *"(setup|connect)"'` finds. A test that then waits on the
old fixed number (`--until made:3`) reads the printed number instead,
or, where that is more than an argv edit, keeps the number the fresh
deployment gives (the first forge made is 1) and says so in the report.

## Steps

T = `temper/crates/temper-client-tools/src`.

1. **`forge setup`** (T/forges.rs, T/table.rs): `forge setup [--org
   ORG] [--repository REPOSITORY] [--project PROJECT] [--playground]
   [--key KEY]`; a positional is a usage error. Prints `setting up forge
   N` (N the answer's number; `setting up` alone when the answer names
   0), then as today, waiting on that number. Its doc lines: "Set up
   Forgejo on this machine as a new forge, with a first repository".
2. **`forge connect`:** `forge connect [FORGE] github|github-app …`:
   with no FORGE a new GitHub (`Connect { forge: None }`), with one go
   on with that kept forge; `forgejo-admin` and `forgejo` keep FORGE
   required (a kept Forgejo). Prints the answered number with
   `connecting`.
3. **`forge remove FORGE [--key KEY]`:** sends
   `forge::remove::ask(forge)` under `--key` or, absent,
   `forge::remove::remove_key(forge)`; prints `removing`; `refused: not
   offered` (exit 2; the doc line: "the playground's forge leaves only
   with the playground: temper ctl playground end"); `unknown forge`
   (exit 2). Doc: "Remove a forge: temper stops using it now, ends its
   work, and erases what it holds for it; an owned Forgejo goes with its
   data."
4. **`playground end [--key KEY] [--timeout DURATION]`:** sends
   `forge::demo::end_ask` under `--key` or `forge::demo::end_key()`;
   `resetting`, then reads the playground every `READ_AGAIN` until none
   (default timeout 300 s; past it `not met: timeout`, exit 3), prints
   `ended`. `refused: not offered` (exit 2: no playground runs). The
   group's line: "The playground: start it, or end it to set temper up".
   `playground start` sends `StartPlayground`, as today otherwise.
5. **`forge list`:** a row not `finished` says `not finished` after its
   connection (draft 6.3), in text and `--json`.
6. **The CLI shell's fixtures** (`temper-client-shell-cli/src/tests/`)
   follow.
7. **Checks:** the tests above; regenerate the help goldens
   (`SKEIN_CODEGEN_WRITE=1` on `the_help_prints_as_its_golden_files`)
   and the two reference pages (above); `cargo fmt`; `heavy cargo
   clippy -p temper-client-tools -p temper-client-shell-cli -p temper
   -p temper-client-world --all-targets -- -D warnings`; one `heavy cargo
   nextest run -p temper-client-tools -p temper-client-shell-cli -p
   temper-client-world -p temper-docs` and `-p temper --lib`.
8. Old forms from `temper/`; commit and push.

## Report

- The tools' final usage lines (D2's and D4's specs name them).
- Each caller edited outside the header, and any that waited on a fixed
  number.
- The hand-written help sentences the lead should change.
