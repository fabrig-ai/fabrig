# inproc-runner: the shapes that cross tasks

The run's binding calls. A later section adds; none changes an earlier one.

## 1. A starved pass in one process (task `starved`; the runner reads it)

The design: `skein/docs/design/testing.md`, 5 (*The real loop classifies its patience*);
`skein/docs/foundation/testing-strategy.md`, 2.10.

- **The variable.** The one-process runner sets `TIERS_ONE_PROCESS=1` in every test
  binary process it starts, and never `NEXTEST`. A process reads either variable by its
  presence (`std::env::var_os(..).is_some()`), whatever its value.
- **`skein_world`, at the crate root** (`pub use deadline::{..}` beside `Starved`):

  ```rust
  /// The unwind payload of a starved pass in a one-process run.
  #[derive(Debug)]
  pub struct StarvedPass;

  /// Whether a starved wait ends its test as a pass here: `TIERS_ONE_PROCESS` or
  /// `NEXTEST` is set.
  #[must_use]
  pub fn starved_passes() -> bool;

  /// Ends the running test with `line`, a `skein-world: starved:` line a helper
  /// process wrote, as `Starved::pass` ends it; elsewhere fails with the line.
  pub fn pass_starved(line: &str) -> !;

  impl Starved {
      /// Ends a test as a starved pass.
      pub fn pass(self) -> !;
      /// Ends a helper process (a fixture binary whose test reads its stderr) as a
      /// starved pass: the line and exit 0 where `starved_passes()`; elsewhere fails
      /// as `pass` does.
      pub fn exit(self) -> !;
  }
  ```

- **How `pass` ends, in this order of precedence:**
  1. `TIERS_ONE_PROCESS` set: writes the line `skein-world: starved: waited ...` (the
     `line()` it writes today, unchanged) and a newline to stderr in one `eprintln!`,
     then `std::panic::resume_unwind(Box::new(StarvedPass))`. `resume_unwind` runs no
     panic hook, so nothing is printed after the line. libtest catches the unwind and
     reports the test failed; the runner reads it starved (below).
  2. `NEXTEST` set: the line, then `std::process::exit(0)`, as today.
  3. Neither: `crate::fail(&format!("skein-world: starved: {}", self.figures))`, as
     today.

  `pass_starved(line)` is the same with `line` in place of `self.line()`; outside both
  it fails with `line`. The caller ends what it owns first, as today: the unwind runs
  the destructors of what is still alive, the exit skips them.
- **Who keys on what.** `real.rs`'s two deadline paths take the starved arm where
  `starved_passes()`; the real conformance's `forward_starved`
  (`skein/tests/conformance/real/tests/suite/signal_sender.rs`) forwards the helper's
  line with `pass_starved` where `starved_passes()`; `signal_sender_fixture`'s
  patience is armed where `starved_passes()` and its starved end is `Starved::exit`.
  `skein/tests/world/tests/suite/real.rs`'s children have both `NEXTEST` and
  `TIERS_ONE_PROCESS` removed.
- **How the runner reads it.** The runner runs every binary with `--nocapture` and its
  stdout and stderr on one pipe, so a test's output is what the process wrote between
  libtest's `test NAME ... ` and libtest's result word for it. A test is **starved**
  when the last line of that output, the result word aside, begins with
  `skein-world: starved:`, whatever libtest's result: recorded starved, a warning,
  never failed and never passed. Output after the line (a panic message, a
  destructor's print) leaves the test failed, as libtest reported it.
