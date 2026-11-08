+++
id = "summary"
part = "summary"
kind = "feature"
needs = []
files = ["tools/tiers/cpu.py", "tools/tiers/test_cpu.py", "tools/tiers/one-process", "tools/tiers/cpu-summary"]
effort = "high"
minutes = 180
+++

# inproc-runner: a one-process run's summary, exit status and command

## Goal

`cpu.py` reads a one-process run's directory into `summary`'s result, with the tests'
CPU null and the processes' CPU summed, and gains `execute_one_process`, which runs
the mode through `tools/tiers/one_process.py`'s `run` (the task `runner`'s, built in
parallel to the same contract) and returns `execute`'s exit statuses;
`tools/tiers/one-process` runs a profile so by hand. A nextest run's summary only gains
`suite.cpu_by` and `processes`.

Acceptance:

- `summary(directory, limits=None, pair=None, expected=None)` reads a directory whose
  `run.json` has `mode` `"one-process"` from its `results.json`, exactly as
  `temper/docs/plan/inproc-runner/notes/contracts.md`, section 3 gives it, from
  section 2's files; a nextest run's and a tools summary gain `suite.cpu_by =
  "test"` and `processes = []`, every other key as now (`tools/tiers/design.md`,
  10.2: a key may be added, none removed or changed in meaning).
- `print_summary` prints a one-process result as contracts section 3 gives it, and a
  nextest one as now.
- `execute_one_process(command, env, directory, profile, limits,
  config=Path('.config/nextest.toml'))` as 10.2 gives it: loads `run` with
  `runpy.run_path(str(ONE_PROCESS))` (`ONE_PROCESS = Path(__file__).with_name('one_process.py')`,
  module level), calls it, then `summary`, prints the summary, appends what it printed
  to `command-output.txt`, and returns 1 over a CPU budget, else 100 when a failure
  stands, else 0 when starved, else 4 for an empty selection under `no_tests`
  `"fail"`, else 0. A `cpu_test_s` in the profile's `limits` raises `ValueError`
  before `run` is called; `run`'s `ValueError` and an interruption pass through.
- `tools/tiers/one-process [--out DIR] [--budgets PATH] PROFILE [NEXTEST_ARGS...]` as
  11.9 gives it, executable, `python3 -I -B` in its shebang as `cpu-summary` has,
  umask 077.
- `cpu-summary DIR [--json]` on a one-process directory prints its summary and exits
  0, 1 or 2 as for a nextest run (it calls `summary`; change it only if it must).
- `test_cpu.py` green with the rows below, each having bitten once, and its existing
  rows unchanged but for the added keys in any whole-dictionary assertion.

## Inputs

- `tools/tiers/design.md`, 10.1 (the two new lines), 10.2 (`execute_one_process`,
  `summary`'s one-process reading, the result table's `suite` and `processes` rows and
  the paragraph *In a one-process run*), 10.3 (the one-process directory), 11.6
  (outcomes), 11.8 (the run directory), 11.9 (by hand); 7.3 (*Budgets and
  completeness*, the verdicts and exit codes).
- `temper/docs/plan/inproc-runner/notes/contracts.md`, sections 2 and 3;
  `temper/docs/plan/inproc-runner/notes/common.md`.

## Owns and may change

- Owns the files in `files`. `tools/tiers/one_process.py` is the task `runner`'s and
  does not exist on your base: never create it. Your tests stand a small stand-in in
  its place, a file in the test's temporary directory defining `run` that writes a
  one-process directory from the contract's shapes (or raises), with `ONE_PROCESS`
  pointed at it through `execute_one_process.__globals__`, as `execute_driver` sets
  `GRACE_S`; for `tools/tiers/one-process`, a temporary copy of `cpu.py`, `run` and
  `one-process` beside such a stand-in.
- `summary` keeps one function and dispatches on `run.json`'s `mode`; the one-process
  reading may live in its own function beside `tools_summary`. `summarize_records`
  may be shared or not; the nextest path's results stay byte for byte as they are but
  for the two added keys.
- `tools/tiers/one-process` finds Cargo metadata's target directory with
  `tools/tiers/run`'s `target_directory` (loaded with `runpy.run_path`, as `cpu-run`
  loads it) and the prefix with its `command_prefix`; `run` itself stays unchanged.
- If a whole-dictionary assertion in `test_cpu.py` breaks only for `cpu_by` or
  `processes`, add the key to its expectation and name the row in the report; any other
  change to an existing row's expectation is a red of yours.

## Steps

1. **Rows**, in `test_cpu.py`, a new `OneProcessTests` class with a helper that writes
   `run.json` and `results.json` from the contract's shapes, the failing ones first:
   - *passed:* `verdict` `no budget set`, `tests` with null CPU, `attempts` 1,
     `suite.cpu_s` the processes' sum (an `alone` process included), `cpu_by`
     `"process"`, `processes` in order with `cpu_s = user_s + system_s`;
     `print_summary`'s text has `CPU not measured` and a process line.
   - *a crash and its fresh process; a failure that passes alone:* `failures` names
     both; the `may depend on an earlier test` warning with its count; `alone` and
     `before` in `tests`.
   - *a hang past the global timeout:* a `cancelled` test counts `starved`, the
     unobserved test is in `unobserved` (`attempt` 1) and `warnings`,
     `suite.starved_run` true, `verdict` `starved`, no failure.
   - *a starved pass:* `starved` 1, no failure, `verdict` `starved`.
   - *a split binary:* two processes of one binary ID, both in `processes`, CPU
     summed.
   - *budgets:* `cpu_suite_s` passed gives the suite's `over_budget` line and
     `verdict` `over budget`; a `cpu_test_s` for the pair raises `ValueError`;
     `wall_hard_s` passed by `wall_s` makes `starved_run`.
   - *invalid*, a table: an unfinished `run.json`; a test both in `tests` and
     `unobserved`; a selected count that disagrees; an unknown outcome or kind; a
     process index out of range; a negative `user_s`; a `cancelled` outcome or an
     unobserved test with `cancelled` null: each `ValueError`, and `cpu-summary` exit 2.
   - *a nextest run and a tools run:* `cpu_by` `"test"`, `processes` `[]`, all else as
     the existing rows have it.
   - *`execute_one_process`*, with the stand-in `run`: exit 0, 100 (a failure), 0
     (starved only, and a cancelled run), 1 (over budget, a failure too), 4 (empty
     selection, `no_tests` `"fail"`), 0 (`"pass"`); the summary printed and appended
     to `command-output.txt`; the stand-in's `ValueError` and `KeyboardInterrupt`
     raised again; a `cpu_test_s` limit refused before the stand-in runs (it records
     nothing).
   - *`tools/tiers/one-process`*, in a temporary copy: the command it gives `run` is
     the prefix (`TIERS_WRAPPER`), `cargo nextest run --profile PROFILE
     --no-fail-fast`, `--workspace` but not when the arguments hold `-p` or
     `--package`, then the arguments; `--out DIR` is used, and without it a fresh
     directory under the target directory is made and printed first (a fake `cargo`
     on `PATH` answering `metadata`); `--budgets` loads limits; its exit status is
     `execute_one_process`'s, and 2 for a `ValueError`.
2. **`cpu.py`**: `cpu_by` and `processes` in the nextest and tools results; the
   one-process reading and its checks; `print_summary`; `ONE_PROCESS` and
   `execute_one_process`.
3. **`tools/tiers/one-process`.**
4. `python3 -I -B tools/tiers/test_cpu.py` (through heavy, `notes/common.md`);
   `python3 -I -B tools/tiers/test_run.py`, unchanged and green;
   `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As the implementer's text gives it, with the rule each row broke and the failure
seen, and any existing row whose expectation gained a key.
