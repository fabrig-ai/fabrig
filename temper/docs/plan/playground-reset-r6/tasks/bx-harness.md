+++
id = "bx-harness"
part = "pg"
kind = "feature"
needs = ["bx-init", "cl-ctl"]
files = ["temper/crates/temper/tests/support/boxed.rs", "temper/crates/temper/tests/support/ctl.rs", "temper/crates/temper/tests/support/ctl/**", "temper/crates/temper/tests/support/forges.rs", "temper/crates/temper/tests/support/mod.rs", "temper/crates/temper/tests/process.rs"]
effort = "high"
minutes = 90
+++

## Goal

The process tier's ctl stories can kill the whole box at a chosen step and start it again
on the same directories, and make, remove and reset forges at runtime through `temper ctl`
(`cl-ctl`'s `forge make`, `forge remove`, `playground reset`), so that draft section 8's
stories (a reset cut by a crash, a set up killed at each step, GitHub gone on after a
restart) can be written. No story of the draft is written here.

Acceptance:
- `support::ctl::Step` gains `KillBox` (SIGKILL to the box and every descendant, through
  the retained process tree as `Ending::Crash` does in `support/mod.rs`; the step ends once
  none is left) and `StartBox` (the box started again with the story's flags and
  directories, as the story's first start; the step ends once serve's link is printed, and
  the ctl client signs in again when the story signed in). A story may hold several of
  each; the referee's evidence (stdout, stderr, left processes) spans every start.
- `support::forges` holds the forge calls of `cl-ctl`: `make_here()`, `make_github()`,
  `make_forgejo(api, git)`, `remove(forge)`, `playground_start()`, `playground_reset()`
  (each a `support::ctl::Call`), and `made(&call) -> u16` reading `made: forge N` from a
  call's output (its JSON form when the call used `--json`).
- One process test proves the two steps:
  `a_box_killed_and_started_again_serves_the_same_store` (sign in; `forge list --json`;
  `KillBox`; `StartBox`; `forge list --json` answers again; nothing left at the end), run
  with `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=a_box_killed_and_started_again_serves_the_same_store)'`.
- The existing ctl stories behave as before (run two of them, below).

## Inputs

- `temper/docs/design/protocol/setup.md`, 13.4 (the process tier).
- Draft `temper/docs/design/draft/playground-reset.md`, section 8 (the process stories).
- `temper/docs/plan/playground-reset-r6/notes/common.md` (process-tier tests run only as a
  spec says).
- `temper/docs/plan/playground-reset-r6/tasks/cl-ctl.md` (the commands' words and outputs).
- The harness as it is: `temper/crates/temper/tests/support/boxed.rs` (`run_ctl`, `Judge`,
  `restart_engine`, `dying_with_test`), `support/ctl.rs` (`Step`, `Call`, `Story`, the
  step loop), `support/mod.rs` (`Ending::Crash`, `signal_tree`), `support/parent.rs`.

## Owns and may change

- Owns: the files in `files` (a new `support/forges.rs` and its `mod` line).
- Not here: any other process test; the document builders (`support/configuration.rs`).
- Options, pre-approved:
  - Where the box's restart lives (`boxed.rs`'s referee or `ctl.rs`'s loop) is inside;
    default: the referee owns the box's process and its restart, the ctl loop asks it.
  - If a kill while a ctl call is in flight needs that call cut first, `KillBox` cuts it as
    `Step::Cut` does and the story's result records it as cut.

## Steps

1. `support/boxed.rs`, `support/ctl.rs`: the two steps; `support/forges.rs`: the calls.
2. `heavy cargo check -p temper --tests`.
3. The new test in `tests/process.rs`, run alone (command above). Then two existing ctl
   stories alone, unchanged:
   `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=rung_b0s_funds_document_applies_through_ctl_and_preserves_the_settings)'`
   and `-E 'test(=temper_box_prints_its_link_and_stops_its_processes)'`. A failure of one of
   these your change cannot reach runs once alone, as `AGENTS.md` 6 says, and is reported.
4. `heavy cargo nextest run -p temper` (default profile); `cargo fmt`;
   `heavy cargo clippy -p temper --all-targets`. Old forms never raised. Commit and push.

## Report

How a story kills and restarts the box (two lines, with an example step list), for the
story tasks.
