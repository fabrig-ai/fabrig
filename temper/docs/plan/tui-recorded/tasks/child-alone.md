+++
id = "child-alone"
part = "tui-approval-flow"
kind = "feature"
needs = []
files = ["skein/crates/skein-io/src/records.rs", "skein/crates/skein-io/src/layer.rs", "skein/crates/skein-io/src/process.rs", "skein/crates/skein-io/src/kernel.rs", "skein/crates/skein-io/src/pipe.rs", "skein/crates/skein-io/src/tests/process.rs", "skein/crates/skein-io/src/tests/kernel.rs", "skein/crates/skein-io/src/tests/records.rs", "skein/crates/skein-shell/src/linux/process.rs", "skein/crates/skein-shell/src/darwin/process.rs", "skein/crates/skein-sim/src/sim/processes.rs", "skein/crates/skein-sim/src/tests/null_descriptor.rs", "skein/docs/design/io.status.md"]
effort = "high"
minutes = 120
+++

## Goal

The two io mechanisms the TUI's opener needs, generic and in skein before their temper consumer (`AGENTS.md`, 1): a child spawned alone, whose group io never signals, and a null descriptor in a spawn's table (`skein/docs/design/io.md`, 6, *A child spawned alone* and *A null descriptor*; 6.3's `Way`; `skein/docs/design/kernel.md`, 6.2). Today closing a child, and releasing one that exited, sends `Kill` to its group (`skein-io/src/process.rs`, `begin_close` and `release`), which would end the browser an opener started; and a spawn's table can give a child only pipes or held descriptors, so an opener's stdout and stderr would be pipes its browser inherits.

Acceptance:

- `io::Request::SpawnAlone { owner, spawn }`: the same fields, checks, refusals and events as `Request::Spawn`; its child records that it is alone. Closing it (`Close` or `Abort`) while it runs sends `Kill` to `Target::Child`, then reaps it as today; once it has exited, closing and the release after its exit send no signal and reap it. Its pipes settle before `Closed` as any child's. An owner's `Signal { to: Group }` still reaches the group. `Request::owner()` and every query on `Request` answer for it as for `Spawn`.
- `kernel::Way::Null`: the child's descriptor opened on `/dev/null`, `O_RDWR`, by the file actions of Linux's `pidfd_spawn` and macOS's `posix_spawn` (`posix_spawn_file_actions_addopen`); no parent end, no token in `Spawned`, nothing held by io; a table naming one child descriptor twice is refused as today, null entries counted. The simulator gives the child a descriptor that reads end at once and takes every write.
- Rows, red first, beside their siblings in `skein-io/src/tests/process.rs` (`owner_signals_target_the_child_or_group_running_and_exited`, `a_child_close_kills_its_group_after_an_in_flight_owner_signal_settles` are the shape): a child spawned alone, closed running, submits `Kill` to `Target::Child` and never to `Group`, then reaps and tells `Closed` once; exited on its own, it is reaped with no signal submitted; closed after an in-flight owner signal, the same once it settles; a `Spawn` child keeps today's group kill (the existing rows unchanged). A row of `given_entries_resolve_without_pipe_entities_and_preserve_pipe_order`'s kind: null entries among pipes and given ones, `Spawned`'s tokens the pipes' alone in order; and a duplicate-descriptor row with a null entry. The simulator's row, in a new `skein-sim/src/tests/null_descriptor.rs` beside `given_listener.rs` (its `mod` line in `sim/processes.rs`): a child with null 0, 1 and 2 reads end and writes succeed.
- Break each rule once by hand (alone closing with `Group`; the release after exit signalling; `Null` given a token), see the row fail, restore; name them in the report.
- Every exhaustive `match` on `io::Request` and on `kernel::Way` in the workspace gains its arm (call sites only, no behaviour: a fixture that refuses spawns refuses `SpawnAlone` alike). Name them in the report.
- `io.status.md`'s *Not yet* entry for these is removed.
- `heavy cargo nextest run -p skein-io -p skein-sim -p skein-shell` passes (the shell's real spawn rows, if any are focused, with them); `heavy cargo check --workspace --all-targets` and clippy for the three skein packages clean.

## Inputs

- `skein/docs/design/io.md`, 6 and 6.3 (*Given descriptors* is the model for `Null`); `skein/docs/design/kernel.md`, 6.2.
- The code: `skein-io/src/process.rs` (`spawn`, `close`, `begin_close`, `release`, `Child`), `skein-io/src/layer.rs` (the request's dispatch), `skein-io/src/records.rs`, `skein-io/src/kernel.rs` (`Way`), `skein-shell/src/linux/process.rs` and `darwin/process.rs` (the file actions), `skein-sim/src/sim/processes.rs`.

## Owns and may change

- Owned: the files above.
- Readers, call sites only: every exhaustive match on `io::Request` or `kernel::Way`, in any tree.
- Options:
  - The child's flag on io's `Child` (default) or its own phase: whichever keeps `begin_close` and `release` one handler each.
  - Whether `SpawnAlone` and `Spawn` share one handler with a parameter (default) or two.

## Steps

1. Read the inputs.
2. Rows as above, red first; break each rule by hand, restore.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy and the tests above.
4. Commit and push.

## Report

Each row and its broken-rule failure; the match sites given arms; how the simulator answers a null descriptor.
