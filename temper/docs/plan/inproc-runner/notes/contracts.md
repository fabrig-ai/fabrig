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

## 2. The one-process run and what it leaves (task `runner` writes, `summary` reads)

The design: `tools/tiers/design.md`, 11 (all of it) and 10.2 and 10.3's one-process
additions.

- **`tools/tiers/one_process.py`** (task `runner`), loaded with `runpy.run_path`:

  ```python
  def run(command, env, directory, profile, limits, config=Path('.config/nextest.toml')):
      """Run one suite a binary to a process (tools/tiers/design.md, 11) into a fresh
      `directory`; write 11.8's files and return None. Refusals raise ValueError
      before anything runs; an interruption stops the lanes and is raised again."""
  ```

  and, as a script, `python3 -I -B tools/tiers/one_process.py lanes PLAN`, the lanes.
  `run` prints the listing's output and each test's status line as they come; it
  prints no summary and computes no exit status: `cpu.py` does.
- **`cpu.py`** (task `summary`) loads it as
  `runpy.run_path(str(ONE_PROCESS))['run']`, with the module-level constant
  `ONE_PROCESS = Path(__file__).with_name('one_process.py')`, so that a test can point
  it elsewhere through the function's `__globals__`, as `test_cpu.py` sets `GRACE_S`.
- **`run.json`**, written by `run` with `save` semantics (one JSON line, atomic):

  ```json
  {"profile": "default", "mode": "one-process", "finished": true, "starved": false,
   "wall_s": 12.5, "returncode": 0, "cancel_code": null, "lanes": 8}
  ```

  `starved` is true when the global timeout or the wall hard limit cancelled the run;
  `returncode` is the lanes' exit status (null when the selection was empty and no
  lanes ran), `wall_s` 0.0 then.
- **`results.json`**, one JSON object, written once after the lanes end:

  ```json
  {
    "profile": "default",
    "selected": 3,
    "no_tests": "fail",
    "cancelled": null,
    "processes": [
      {"binary_id": "pkg::suite", "kind": "tests", "tests": ["a", "b"],
       "user_s": 0.11, "system_s": 0.02, "wall_s": 0.4, "start_s": 1234.5,
       "exit_code": 101, "signal": null, "leaked": false}
    ],
    "tests": [
      {"binary_id": "pkg::suite", "test_name": "a", "outcome": "passed",
       "wall_s": 0.01, "process": 0, "before": [], "alone": null, "output": null},
      {"binary_id": "pkg::suite", "test_name": "b", "outcome": "failed",
       "wall_s": 0.02, "process": 0, "before": ["a"], "alone": "passed",
       "output": "thread 'b' panicked at ..."}
    ],
    "unobserved": [{"binary_id": "pkg::suite", "test_name": "c"}]
  }
  ```

  - `selected`: the selection's count; `tests` and `unobserved` together hold each
    selected test exactly once. `no_tests`: `fail`, `warn` or `pass`, the empty
    selection's rule (11.2).
  - `cancelled`: null, `"global-timeout"` or `"wall-hard"`.
  - A process: `kind` `"tests"`, `"fresh"` (after a crash) or `"alone"` (a
    diagnostic); `tests` the names libtest announced, in their order; `user_s`,
    `system_s` from `wait4`'s rusage, `wall_s` from its start to its reaping,
    `start_s` host monotonic seconds; exactly one of `exit_code` (0 to 255) and
    `signal` is non-null, as in 7.2's records; `leaked` per 11.4.
  - A test: `outcome` `"passed"`, `"failed"`, `"crashed"`, `"starved"` or
    `"cancelled"` (11.6); `process`, the index of the `"tests"` or `"fresh"` process
    it ran in; `before`, for a failed or crashed test the names that ran before it in
    that process, else `[]`; `alone`, for a failed or crashed test `"passed"` or
    `"failed"` when its diagnostic ran, else null; `output`, its output as text
    (UTF-8, `backslashreplace`) for every outcome but `"passed"`, else null. A test's
    alone rerun is a process of kind `"alone"` holding only it; the test's entry stays
    the first run's.
- **`plan.json`**, `junit.xml`, `command-output.txt` and `starved.json` are the
  runner's, as 11.4 and 11.8 give them; `summary` reads none of them in a one-process
  directory. `cpu.py`'s `execute_one_process` appends its printed summary to
  `command-output.txt`.

## 3. What `summary` gives for a one-process run (task `summary`)

As `tools/tiers/design.md`, 10.2 gives it: the same keys as a nextest run, and
`processes` and `suite.cpu_by`. From `results.json`:

- `tests`: one entry per entry of `results.json`'s `tests`, sorted by `(binary_id,
  test_name)`: `binary_id`, `test_name`, `attempts` 1, `cpu_s`, `user_s`, `system_s`
  null, `wall_s`, `starved` (1 for `"starved"` and `"cancelled"`, else 0), `leaked`
  and `incomplete` null, `alone`, `before`.
- `suite`: `tests` and `attempts` the count of `tests`; `cpu_s`, `user_s`, `system_s`
  the sums over every process, `"alone"` ones included; `wall_s` `run.json`'s;
  `test_wall_s` the sum of the tests' walls; `starved` the starved and cancelled
  count; `starved_run` `run.json`'s `starved`, or the wall hard limit passed (7.3);
  `unobserved` the count; `leaked` and `incomplete` null; `cpu_by` `"process"`.
- `processes`: one entry per process in `results.json`'s order: `binary_id`, `tests`,
  `cpu_s` (`user_s + system_s`), `user_s`, `system_s`, `wall_s`, `exit_code`,
  `signal`, `kind`, `leaked`.
- `failures`: `"BINARY_ID TEST_NAME"` for each `"failed"` and `"crashed"` test,
  sorted.
- `unobserved`: `{binary_id, test_name, attempt: 1}` for each, sorted.
- `warnings`, in this order: for each failed test whose `alone` is `"passed"`,
  `"{pair} {binary_id} {test_name}: may depend on an earlier test: it passed alone
  after {n} tests in its process"`; for each leaked process, `"{pair} {binary_id}:
  a process left a descendant holding its output"`; for each unobserved test, `"{pair}
  {binary_id} {test_name}: unobserved, the run was cancelled before it"`.
- `over_budget`: the suite's line when `cpu_suite_s` is passed, as 7.3's; a
  `cpu_test_s` in the pair's table raises `ValueError`.
- `verdict`: as `summarize_records` gives it: `over budget`, `starved` (a starved
  run, or a starved or cancelled test), `within budget`, `no budget set`.
- Invalid (raises `ValueError`): `run.json` unfinished or of another mode;
  `results.json`'s tests and unobserved not each selected test once, or `selected`
  disagreeing; an outcome, kind or index out of its set; a non-finite or negative
  figure; a `"cancelled"` outcome or an unobserved test in a run whose `cancelled` is
  null.
- **A nextest run's and a tools summary** gain `suite.cpu_by = "test"` and
  `processes = []`, and nothing else changes.
- **`print_summary`** prints, for a one-process run, a line per process
  (`{binary_id} process {kind}: CPU {cpu_s:.6f}s (user .. system ..) wall .. tests {n}
  {exit status}`) after the tests' lines, with `CPU not measured` in place of a test's
  CPU, and the `may depend` labels among the warnings; a nextest run prints as now.
