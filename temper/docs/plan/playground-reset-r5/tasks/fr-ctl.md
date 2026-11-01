+++
id = "fr-ctl"
part = "forge-records"
kind = "feature"
needs = ["fr-wire", "cl-tools"]
files = ["temper/crates/temper-client-tools/src/forges.rs", "temper/crates/temper-client-tools/src/forges/**", "temper/crates/temper-client-tools/src/tests/forges.rs", "temper/crates/temper-client-tools/src/tests/forge_connection.rs"]
effort = "high"
minutes = 75
+++

## Goal

`temper ctl` names a new forge, so the process tests and a person at a
terminal make forges as the welcome does (draft
`temper/docs/design/draft/playground-reset.md`, section 3): `forge setup`
without a number sets up a new owned Forgejo, and `forge connect new ...`
connects a new GitHub or a Forgejo elsewhere, each sending
`ForgeChoice::New` (`notes/forge-records.md`, section 6), then following
the forge the engine made by its row in the Forges watch.

Acceptance:

- `forge setup [FORGE] [--org ..] [--repository ..] [--project ..]
  [--playground]`: `FORGE` optional; without it the request names
  `new { owned }`, with it `kept { forge }` (today's). Its wait follows
  the forge whose row appears in the Forges watch after the answer
  (`setting_up`): the one number not listed before the request; then as
  today (made, adopted, `not met: timeout`).
- `forge connect FORGE way ...`: `FORGE` is a number (today's) or `new`;
  `new` with way `app`/`manifest` names `new { github { read_only } }`
  (a `--read-only` switch, off by default), with way `admin` names `new {
  forgejo { address } }` from a required `--address URL`. Its wait and
  its printed link follow the new forge's row as `forge setup` does.
- `forge remove FORGE`: sends `remove_forge { forge }`, prints `removing
  FORGE` on `removing`, `refused: no such forge` (exit 2) on
  `remove_refused`; it waits up to 60 seconds for the forge's row to leave
  the watch, else `not met: timeout` (exit 3). (For part `remove-forge`;
  the service serves it from `rf-service`.)
- `make_refused` prints `refused: full`, `refused: taken` or `refused:
  not offered` (exit 2).
- Tests (the crate's library tests): `src/tests/forges.rs`
  `setup_without_a_number_sends_a_new_owned_choice_and_follows_the_new_row`,
  `a_refused_make_prints_its_reason`, `remove_prints_removing_then_waits_for_the_row_to_leave`;
  `src/tests/forge_connection.rs`
  `connect_new_manifest_sends_a_new_github_choice`,
  `connect_new_admin_requires_an_address_and_sends_a_forgejo_choice`.
  `src/tests/printed.rs`'s `every_printed_call_parses` passes.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 3),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 2 and 6).
- `temper/docs/design/client/tools.md` if it exists, else
  `temper/docs/design/client/README.md`, the section on `temper ctl`
  (`rg -n 'ctl' temper/docs/design/client/README.md`).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s1b-connector-store-acts.md`,
  section 3, "CLI/tool senders".

## Owns and may change

- Owns: `temper-client-tools`' `src/forges.rs`, `src/forges/`,
  `src/tests/forges.rs`, `src/tests/forge_connection.rs`. The client
  part's tasks own the rest of `temper-client-tools` and never these files
  while this task runs.
- Not yours: `temper/docs/help/` (a lead's). If a help or docs test reads
  the changed forms and fails, report it under `## Left` with the page's
  path and the form it needs.
- Options: if the Forges watch is not open in a call's run, open it as
  `forge list --until` does; if a new forge's number cannot be told from
  the rows (two made at once by two people), take the highest new number
  and say so.

## Steps

1. The forms and their requests; the follow of the new row.
2. `forge remove`.
3. Tests. Narrow: `heavy cargo nextest run -p temper-client-tools --lib -E
   'test(/forges/) | test(/forge_connection/) | test(/printed/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-tools --all-targets
   -- -D warnings`; focused tests of `temper-client-tools`; `heavy cargo
   check -p temper -p temper-client-shell-cli --tests`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says; each form's printed usage line.
