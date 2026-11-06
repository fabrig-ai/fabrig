+++
id = "box-link-stories"
part = "box-stories"
kind = "feature"
needs = ["box-readers-early"]
files = ["temper/crates/temper/tests/suite/box_listeners.rs", "temper/crates/temper/tests/process.rs", "temper/crates/temper/tests/fake_backends/**", "temper/crates/temper/tests/support/boxed.rs"]
effort = "high"
minutes = 60
+++

# The real binary's stories: block, early open, restart

## Goal

The stories that only a real box can show, on top of the lower tiers' tests: the terminal's bytes, a link opened at once, a killed serve's fresh link, and `--fake-backends`' lines after the block.

Paths `tests/...` and `suite/...` here are under `temper/crates/temper/tests/`, or that directory's parent where they begin `tests/`.

Acceptance, each `temper box` real, no `--echo`:

- `suite/box_listeners.rs`, `a_fresh_box_prints_its_block_and_nothing_else`: stdout is one line, the link. Stderr is exactly `PRELUDE` and `FOOTER`, written out as literals here, with no other byte. After a stop, `box.log` holds in order `First run: `, `Logs: `, `Ctrl-C stops temper.`, `Starting serve`, `Stopping`, `Stopped`, each after `HH:MM:SSZ `, and holds no link.
- `suite/box_listeners.rs`, `the_first_link_opened_at_once_waits_and_signs_in`: a `TcpStream::connect` to the link's address, as soon as the link is read, succeeds, whether or not serve listens yet. `support::terminal::open` given the link then signs in as the owner. Record in its output whether `serve.log` had `listening clients ` at the connect, printing no secret.
- `suite/box_listeners.rs`: the secret given is the first link's: that first open is a sign-in with the very link the block printed, and stdout has no second link after it.
- `tests/process.rs`, `a_killed_serve_keeps_its_port_and_prints_a_fresh_link`: kill serve alone. While it is down, a connect still succeeds (the socket is held). Its restart prints a blank line on stderr and one link on stdout, unprefixed and untimed. That link signs in. The block's first link, opened after the restart, is refused, so it is not replayed. A process found by `find(live_pids, ...)` may be Linux's alone: keep that test `#[cfg(target_os = "linux")]` as `process.rs` keeps its others.
- `tests/fake_backends`: one existing start, its stderr, has the block first and then `box: effective configuration `, in that order, and no `Logs:`.
- A taken port stays as `suite/box_listeners.rs`, about lines 103-115, has it: `cannot listen`, empty stdout, no block.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.2, *The sign-in block*; 13.4, *`temper box`*.
- `temper/docs/design/draft/box-link-first.md`, sections 2-4.
- `skein/docs/foundation/testing-strategy.md`, 8.1.
- `temper/docs/plan/box-link-first/notes/contracts.md`.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns the files in the header; of `tests/support/boxed.rs`, only `run_ctl_peers`' readiness path (see Answers).

**Over budget is red.** A real box's start is seconds of CPU, and `--test suite` is the focused tier. Measure each new test's CPU as heavy reports it. Prefer one box per test, stopped as soon as it has shown its fact. If a suite story puts its tier over budget, answer your text's three questions in order, the third with measurements. Moving it to `process-temper` only to escape the budget is forbidden.

Options, each pre-approved:

- `the_first_link_opened_at_once_...` and "the secret given is the first link's" may be one test, if it reads as both.
- Read `support::terminal::open` for its private directory: any scratch directory the test owns will do.
- If `serve.log` already had the line at the connect, the test still passes: being early is the machine's luck, and the lower tier pins the order (`box-first-block`).

## Steps

Write each story with its acceptance's name. Each must bite: for the block, break a word of `PRELUDE` by hand; for the early open, make `run_configuration` skip the listeners for one run and see the connect refused; for the restart, give the secret on every spawn and see the replay caught. Restore each. Run each alone, then fmt, clippy on `temper`. Commit and push.

## Report

As your text says, with each story's CPU, and whether each early open was early.

## Answers

2026-11-06. **Repair the fake-backend readiness read here, or have `box-readers-early` supply it?** Repair it here: `box-readers-early` has landed on the trunk and this task's `tests/fake_backends` acceptance cannot hold without the repair, so `tests/support/boxed.rs` joins this task's files, for this one change.

- In `run_ctl_peers`, about line 989, when the flags hold `--fake-backends`, `data` is `<--data>/box/fake-backends/data`, the effective data directory the box gives serve (`temper/crates/temper/src/fake_backends.rs`, `directories`); otherwise it stays the `--data` flag. Its two readers, `CtlJudge`'s `data` and `Calls::with_serve_data`, then read this run's `serve.log`. `fake_backends` is computed about line 1010: move it up. `serve_listening` keeps its signature, and nothing else in `boxed.rs` changes.
- No bound, backoff or `BOX_DEADLINE` changes, and no symlink or other workaround.
- Then, with `heavy`: run `fake_backends::walk::chat_document::a_chat_document_is_written_and_revised_on_the_fake_box` alone, which reaches the assertions you added in `walk.rs`, then every `fake_backends::` test in `--test process` under `process-temper`, since the gate's change reaches them all. Stories without the flag read the same path as before. Report the walk's CPU and wall time.
- The commit you pushed stays; add this one on top of it. The three biting checks already ran and need no repeat.
