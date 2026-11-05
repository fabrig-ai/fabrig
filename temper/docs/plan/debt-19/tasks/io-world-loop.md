+++
id = "io-world-loop"
part = "io-worlds"
kind = "feature"
needs = []
files = ["skein/testing/skein-world/src/world.rs", "skein/testing/skein-world/src/referee.rs", "skein/tests/world/tests/suite/**", "skein/tests/io/src/world.rs", "skein/tests/io/src/joined.rs", "skein/tests/io/src/referee.rs"]
effort = "high"
minutes = 75
+++

# skein: the socket and either worlds run on `skein_world`'s loop

## Goal

The first of three steps of `skein/docs/development/debt/D032-world-harness-copies.md`. `World::run` and `either` stop running loops of their own and run on `skein_world`, each process's `Proc::iterate` unchanged inside a `Host`. Every schedule holds, and a digest proves it.

Acceptance:

- `rg -n 'advance_to' skein/tests/io/src/world.rs skein/tests/io/src/joined.rs` finds nothing (`skein/tests/io/src/output.rs` is `io-output-loop`'s).
- The digest of `temper/docs/plan/debt-19/notes/contracts.md`, 2 is the same at your base and at your tip.
- `heavy cargo nextest run -p skein-io-world -p skein-world -p skein-world-tests` passes, all 31 tests of these two worlds with their assertions unchanged. `Proc::iterate`'s signature and `World`'s public API stay, so `suite/ring.rs` and `suite/scenarios.rs`, which call them directly, don't change.
- A new test in `skein/tests/world/tests/suite/` (package `skein-world-tests`, where the harness's own stories live) pins `Referee::idle`: a referee that passes only in `idle` ends the world. By hand: remove the call from the loop, and it fails for that reason. Restore it.
- Net test lines in `skein/tests/io` fall (estimate -25); `skein-world` gains at most 10 production lines (estimate +8), the only production change.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `temper/docs/plan/debt-19/notes/contracts.md`, 2: binding
- `skein/docs/development/debt/D032-world-harness-copies.md`
- `skein/docs/design/testing.md`, 2 and `skein/docs/design/io.md`, 8
- `skein/tests/io/src/processes.rs`, about line 212: the target shape, an `impl Host` over hand entries

## Owns and may change

- **Owns:** the files in `files`.
- **`Referee::idle`, an addition with a default body:**

  ```rust
  /// The world is idle: nothing runs before the clock moves. A referee may change here what it will answer.
  fn idle(&mut self, _now: Time, _procs: &[P], _settled: bool) {}
  ```

  The harness calls it after its busy check and before it reads `settled && passed`, with `settled` read as that check reads it. About 130 implementors don't change. tests/io's referee needs it because `settled` and `overdue` take `&mut self` and call `absent`, which meets an optional expectation for a connection never made: `passed()` can only turn true at idle.
- **The directory needs no harness change:** a `Hosted { proc, directory: Rc<RefCell<Directory>> }`, world-side, borrows it around `iterate`, so a dial sees a `Listening` of the same iteration, as today. `Proc::iterate` and `Owner` don't change.
- **Traps, each read from the code:**
  - `Host::next_mechanism_deadline(&self)` has no `now`: use `owner.next_deadline(self.env.now)`.
  - `check_teardown` fails a world on an owner's deadline after the last word, which the hand loops never checked. State the owner's deadlines as policy and return `LastWord::After(END)`; don't report them as mechanism.
  - `either`'s referee has `next_deadline` of `None`: any wake moves the clock and changes a trace's times. Keep its `now < END` assert, in `observe`.
  - `marks` come from `run_with_inputs`, which lends `&mut Sim` before each iteration; `run_with_inputs` consumes the harness and returns its `Outcome`, so `World::run` maps that to the `Outcome` it returns today.
- **If the digest differs, stop:** contracts, 2.
- **A public addition to `skein_world` beyond `idle` is a question.**

## Steps

1. The probe, at your base: save its output outside the tree.
2. `idle`, its test, its bite.
3. `Hosted`; an adaptor referee over tests/io's `Referee`; `World::run` on the harness; then `either`.
4. The probe at your tip; `diff`. Delete the probe.
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-world -p skein-world-tests -p skein-io-world --all-targets -- -D warnings`; the tests above; `heavy cargo check --workspace --tests`, since 96 packages read `skein-world`. The citations check. Commit and push.

## Report

The common note's items, the number of lines the two digests hold and that they're equal, what you broke and what failed, and the net lines.
