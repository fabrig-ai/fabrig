+++
id = "cl-pg-goes"
part = "pg"
kind = "migration"
needs = ["cl-pg", "cl-ctl"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/crates/temper-client-tools/**",
  "temper/tools/gallery/**",
  "temper/tests/client/**",
  "temper/docs/design/client/journeys/*/0*.html",
]
effort = "medium"
minutes = 90
+++

## Goal

What goes from the client with the demo's end (draft
`temper/docs/design/draft/playground-reset.md`, 6.3, "What goes"; `notes/contracts.md` T6,
"What goes"): the model's and forge's halves, the retired playground, the setup strip, the
demo banner's Connect links, and the client's removal of the scripted account. The
playground is running, leaving or absent; a real account and a real forge never end it.

- Gone from `temper-client-domain`: `forge::demo::{Halves, halves, current, observe_end,
  remove, RetiredDemo}`, `DemoState::Ended`, the demo state's `ended`, `model_ended`,
  `forge_ended`, `deployment` (the cleanup's watch) and `settled` where only the cleanup
  read it, `Side::retired_demo`, the retirement in `Side::serves_pages` (the playground's
  forge serves no pages while the playground runs or leaves), `pages`' `retired_project`
  and `demo_model`, `listed::{Setup, setup}`, and every caller. The client sends no
  `AccountChange` removal of the scripted account, ever.
- `Domain::demo()` returns the running playground's answer (`Option<&forge::Demo>`), no
  halves; its readers (chat suggestions, project navigation, Accounts' playground section,
  `listed::playground`) read it.
- `welcome::model_usable` still leaves the playground's provider out (the scripted model is
  no account of the person's), reading the provider from the answer alone.
- Gone from `temper-client-view`: the setup strip (`frame.rs`, its words `Setup*`), the
  demo banner's halves wording and its Connect links (`DemoBoth`, `DemoModel`,
  `DemoForge`, `setup_underway` if nothing else reads it), the words of a playground ended
  by another forge ("Ended with the playground" and kin, if the client has any).
- The tests of removed behaviour removed (each listed in the report; `tests/demo.rs`'s
  cleanup and retirement cases, `demo_start.rs`'s strip cases, the client world's
  `a_model_connected_removes_the_scripted_account_and_leaves_the_playground_forge_banner`,
  `a_forge_connected_clears_the_demo_everywhere_and_welcome_moves_when_a_model_is_usable`,
  `qa_stale_demo_read_cannot_restore_a_retired_playground`), and three added (step 2).
- The journeys replay without going stale.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T6.
- `temper/docs/design/client/README.md`, 4.8 (what goes), 5.6.
- The draft, 6.1 and 6.3; `tasks/cl-pg.md` (what this task found in place).

## Owns and may change

Owned: `temper/crates/temper-client-domain/`, `temper/crates/temper-client-view/`,
`temper/tools/gallery/`, the joined client world `temper/tests/client/` (removed cases and
the readers of removed items), and `temper/crates/temper-client-tools/` where it reads a
removed item. The shells where the compiler requires. The journeys' printed pages by the
reprint alone.

Options, pre-approved:

- If `RetiredDemo`'s project filter is read by project navigation for a reason other than
  the retired playground (a project the engine still lists), keep the navigation's
  behaviour for a live playground and drop the retired case; say so.
- Renaming the client's `Demo`/`DemoState` vocabulary to the playground's is not this
  task's: leave the names.

## Steps

1. Remove the items of the Goal, following the compiler from `forge/demo.rs` outward.
2. Tests (domain `src/tests/demo.rs`, view `src/tests/demo.rs`):
   `a_real_account_never_removes_the_scripted_account`,
   `a_real_forge_connecting_leaves_the_playground_running`,
   `the_playground_forge_serves_no_pages_while_it_runs_or_leaves`.
3. `heavy cargo nextest run -p temper-client-domain --lib`, `-p temper-client-view --lib`,
   `-p temper-client-gallery --lib`, `-p temper-client-tools --lib`, `-p
   temper-client-shell-web --lib`, `-p temper-client-shell-terminal --lib`, `-p
   temper-client-world --test suite`, `-p temper-client-journeys --lib` (stale stops the
   task; a page's bytes alone, reprint). Compile the world's `qa_all` and `qa_model`
   binaries (`heavy cargo check -p temper-client-world --tests`). `cargo fmt`; clippy
   `--tests` and the wasm clippy of domain and view; old forms (lower counts with `--write`
   if this removes any); commit and push.

## Report

- Every removed test, by file, and the item it tested.
- Under `## Left`: design text naming the halves, the strip or the retired playground
  (`temper/docs/design/client/README.md` 4.8, 5.6; `temper/docs/design/playground.md` 6).

## Answers

2026-11-01, from run `cl-pg-goes-1`.

- **Question:** removing the cleanup watch makes `welcome-playground`'s replay stale (turn
  12, recording line 55: the deployment watch no longer asked); refresh the recording
  elsewhere, or re-record it here? **Answer:** neither here. Do not re-record or edit the
  recording: `cl-journeys`, which already comes after this task (through `cl-world-forges`
  and `cl-world-pg`), records `welcome-playground` again with `again` on the run's code, and
  the run's full checks run once at the end, after it. For this task the stale journey is
  accepted on one condition: `temper-client-journeys --lib` fails only on
  `welcome-playground`, only at that removed watch (`the_committed_journeys_are_their_print`
  and `a_reprint_keeps_a_journeys_readme`, nothing else stale). Quote the stale turn in the
  report and list it under `## Left` for `cl-journeys`. Every other journey still replays;
  reprint any whose pages alone changed. Then finish step 3's other checks (tools,
  shell-web, shell-terminal, the client world's suite, `heavy cargo check -p
  temper-client-world --tests`, clippy `--tests` and the wasm clippy of domain and view),
  and rerun domain and view in full.
- **The gallery's `chats-playground-leaving` failure** (this task's own): keep `cl-pg`'s
  shape. `Domain::demo()` returns the playground's answer whenever the read holds one, while
  it runs and while it leaves (an end in flight, or `resetting`); leaving does not hide it.
  So the leaving chats page keeps its suggestions, as the scene expects. What a leaving
  playground must not show stays on `cl-pg`'s gates, `Side::playground_running()` and
  `Side::leaving()`: the banner's status line, and the playground's forge serving no pages.
  Do not change the scene or its expectation.
