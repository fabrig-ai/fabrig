+++
id = "cl-tools"
part = "client"
kind = "feature"
needs = ["cl-play-domain"]
files = ["temper/crates/temper-client-tools/**"]
effort = "medium"
minutes = 60
+++

## Goal

Move `temper ctl`'s playground tools to the playground's wire (draft
`temper/docs/design/draft/playground-reset.md`, 6.1 and 6.2):
`playground start` sends `StartPlayground` (was `StartDemo`), and a new
`playground end` sends `EndPlayground` and returns once the reset
answers, so the process tier (task `pg-process`) can drive a first run
into the playground and out by its reset.

Acceptance:

- `forges::DEMO_START` becomes `forges::PLAYGROUND_START` (name
  `playground start`, unchanged words but "this deployment has no
  playground" becomes "a forge is kept: the playground is not offered"),
  its record `playground-start` (was `demo-start`); `src/demo.rs` becomes
  `src/playground.rs` printing `StartPlayground`'s outcomes as today's.
- `forges::PLAYGROUND_END`: `playground end [--key KEY]`, line "Leave the
  playground: delete its project, remove its forge and its account",
  prints `reset`, record `playground-end`, returns once the outcome is
  `reset`; `unknown` prints as other tools print it. In the `playground`
  group of `table.rs` after `playground start`.
- Every reader of `w::Request::StartDemo` in this crate reads
  `StartPlayground`/`EndPlayground` (stored-request classification,
  `forges/connection.rs`); `temper_client_domain::forge::playground::ask`
  (or the name `cl-play-domain` gave it) replaces `demo::ask`, and the end
  has its own ask beside it if `cl-play-domain` exposed one, else build
  the `Requesting` record here as `connection.rs` builds the others.
- Tests in the crate's library tests (`src/tests/forges.rs`):
  `playground_start_answers_print_their_words_json_and_status` (was
  `demo_start_...`), `playground_start_keeps_drawn_keys_and_recovers_its_own_answer`,
  `playground_end_prints_reset_and_waits_for_its_answer`; the help
  goldens (`src/tests/golden/`) rewritten with `SKEIN_CODEGEN_WRITE=1` as
  `tests/help.rs` says, and its `USAGES` list given the new usage.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 3),
  `temper/docs/plan/playground-reset-r5/notes/playground.md` (section 5).
- `temper/docs/design/client/tools.md`, the forge tools' section (read
  `rg -n 'playground' temper/docs/design/client/tools.md`).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 3, "CLI consumer", and section 5's `temper-client-tools` lines.

## Owns and may change

- Owns: `temper/crates/temper-client-tools/**`.
- Not yours: `temper/docs/help/reference/ctl-playground.md` (a help page:
  the lead's). If a test compares it with the table, leave that test
  failing and list it under `## Left`; say under `## Design` the page's new
  text as the table renders it.
- If `cl-play-domain` already moved this crate's call sites to compile,
  build on what it did.

## Steps

1. Rename and rewire `playground start`; add `playground end`.
2. Tests; `heavy cargo nextest run -p temper-client-tools --lib -E
   'test(/playground|help/)'`.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-tools --all-targets
   -- -D warnings`; `heavy cargo check -p temper --tests` (the `temper`
   crate's process tests name `playground start`); the focused tests of
   `temper-client-tools` and `temper-client-shell-cli`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says.
