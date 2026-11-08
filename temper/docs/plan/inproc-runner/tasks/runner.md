+++
id = "runner"
part = "runner"
kind = "feature"
needs = []
files = ["tools/tiers/one_process.py", "tools/tiers/test_one_process.py", "tools/tiers/fixtures/one-process/**", ".config/fixture-readers.toml"]
effort = "high"
minutes = 300
+++

# inproc-runner: the one-process run, a binary's tests in one process

## Goal

`tools/tiers/one_process.py`'s `run` takes the caller's nextest command, lists and
builds through nextest under the caller's prefix, and runs each binary's selected
tests in one process, binaries in parallel on as many lanes as CPUs, under that prefix
once for the whole run; it measures each process's CPU from `wait4` and no test's,
and leaves `run.json`, `results.json`, `junit.xml`, `command-output.txt` and
`plan.json` in a fresh directory. The summary and the exit status are another task's
(`summary`): this one ends at the files.

Acceptance:

- `run(command, env, directory, profile, limits, config=Path('.config/nextest.toml'))`
  and the lanes, `python3 -I -B tools/tiers/one_process.py lanes PLAN`, as
  `tools/tiers/design.md`, 11.1 to 11.8 give them, with `run.json` and `results.json`
  exactly as `temper/docs/plan/inproc-runner/notes/contracts.md`, section 2 gives them.
  Nothing of nextest's path changes: `cpu.py`, `run`, `cpu-run`, `cpu-summary` and
  `cpu-wrapper.c` stay as they are.
- `tools/tiers/test_one_process.py`, beside `test_cpu.py` and in its manner
  (`unittest`, temporary directories under the checkout, module setup that builds the
  native fixtures once and prints their build CPU), green with every row below, each
  having bitten once (`notes/common.md`).
- `.config/fixture-readers.toml` gains the entry for the new fixtures:
  `paths = ["tools/tiers/fixtures/one-process/*"]`, `reader = [{ suite =
  "tools/tiers/test_one_process.py" }]`, with a comment beside it as its neighbours
  have.
- One real package's focused suite through `run`, from the slot, the prefix `heavy`:
  `skein-lib` (`['heavy', 'cargo', 'nextest', 'run', '-p', 'skein-lib', '--profile',
  'default']`), every test that `heavy cargo nextest run -p skein-lib` passes passed
  in `results.json`, none failed, the directory's files as 11.8 gives them. Compare
  outcomes and counts only; read and compare no CPU figure. The few lines that call
  `run` for it live untracked in the slot and are deleted before the push; the report
  gives the counts.
- `tools/tiers/design.status.md` stays as it is: the plan's last task, `close`,
  deletes its entries once every part has landed.

## Inputs

- `tools/tiers/design.md`, 11 (all of it); 10.3 (the one-process directory); 7.3
  (*`execute` starts every command as the leader of its own session* to *raised
  again*: the cancellation and interruption the wall hard limit follows); 7.4 (how
  `test_cpu.py` builds its native fixtures); 9 (fixture readers); 3 (`TIERS_WRAPPER`,
  read only to know the prefix is the caller's).
- `skein/docs/design/testing.md`, 5, the bullet *The real loop classifies its
  patience*: what a starved pass writes in a one-process run.
- `temper/docs/plan/inproc-runner/notes/contracts.md`, sections 1 and 2;
  `temper/docs/plan/inproc-runner/notes/common.md`.
- Read, to match: `tools/tiers/cpu.py` (`execute`, `stop`, `session_processes`,
  `save`, `budgets`), `tools/tiers/run` (`list_tests`: how the listing's JSON is found
  among a prefix's lines), `tools/tiers/test_cpu.py` (fixtures, drivers, timeouts).

## Owns and may change

- Owns the files in `files`. `cpu.py` is the task `summary`'s and `run`, `cpu-run`,
  `cpu-summary` nobody's in this plan: read them, load them with `runpy.run_path` and
  call their helpers (`save`, `stop`, `session_processes`, `signal_group`, `GRACE_S`,
  `budgets`) where one fits, and change none of them. A helper you would want changed
  is copied into `one_process.py` instead, and the report says which.
- The sketches bind the names in the design and the contracts. Inside them the
  choices are yours: the lanes' event lines (`tiers-one-process: ` and a JSON object,
  11.4) are between your two halves, and `plan.json`'s fields are yours.
- Options, each pre-approved:
  - Module-level knobs the tests set through `run.__globals__` (as `test_cpu.py` sets
    `GRACE_S`) and that `run` writes into `plan.json`: `LANES` (None: the lanes count
    `os.sched_getaffinity(0)`), `TERM_GRACE_S` (10.0, nextest's grace at the global
    timeout), `WALL_GRACE_S` (15.0, 7.3's), `LEAK_S` (0.1). No environment variable
    or command-line option changes them.
  - Spawn each binary with `subprocess.Popen` (`process_group=0`, `pass_fds`) or
    `os.posix_spawn`, but reap it with `os.wait4` alone: a `Popen` stays referenced
    until `wait4` has reaped it, and its `returncode` is then set from the status, so
    that `Popen`'s own polling can never reap it first.
  - If libtest's announcement or log differs from 11.6's on the toolchain (a
    `--logfile` line with more than `RESULT NAME`, an announcement not flushed before
    the test), follow what the fixture binaries show and say so in the report, with
    the lines seen.
  - A variable nextest gives a test that 11.5 doesn't name, which a test needs, is
    added and named in the report.
  - If the gate's fixture-reader check reads `tools/tiers/fixtures/*` as covering
    `one-process/` too, narrow that entry's paths to `tools/tiers/fixtures/cpu-*.c`.
  - If `heavy` refuses the lanes' command (`heavy python3 ...`), stop: that is a
    `Question:` (the lanes must run under the caller's prefix).

## Steps

1. **The fixtures.** `tools/tiers/fixtures/one-process/`: a few Rust test sources,
   standard library only, built by the suite with `rustc --edition 2024 --test SRC -o
   OUT` (and `--cfg` for variants where that saves a source); a missing `rustc` fails
   the suite with a clear error. A fake `cargo` (Python, run as `python3 -I -B`, with
   a shebang the fixture checkout's command names by path) that answers `nextest list
   --message-format json ...` from a listing the test writes beside it (the built
   binaries' paths, `cwd`s, `filter-match`es, `rust-build-meta` with
   `target-directory`, `non-test-binaries`, `build-script-out-dirs`,
   `linked-paths`), answers `metadata --format-version 1 --no-deps`, and appends its
   argv to a file the test reads. A fake prefix script that appends its argv to a
   file and execs the rest. Each row's fixture checkout is a temporary directory with
   its own `.config/nextest.toml`.
2. **The rows**, in `test_one_process.py`, the failing ones first:
   - *A passing binary:* every test passed, one process of kind `tests` with finite
     `user_s` and `system_s`, `run.json`'s `mode` and `lanes`, a `testcase` per test
     in `junit.xml` with `classname` the binary ID, `plan.json`, and no other file
     in the directory. The prefix ran twice: the listing and the lanes. The listing's
     argv is `nextest list --message-format json --profile PROFILE` and the run's
     selection, with `--no-fail-fast` dropped.
   - *A crash:* tests `a` (passes), `b` (`std::process::abort()`), `c` (passes), in
     one binary: `b` crashed with signal 6 and its output, `c` passed in a process of
     kind `fresh`, and `b`'s rerun is a process of kind `alone` holding only `b`,
     whose `alone` is `"failed"`. A binary that aborts before its first test fails
     all its tests and starts no fresh process.
   - *A hang past the global timeout:* `a` passes, `b` sleeps forever, `c` follows;
     `global-timeout = "1s"`, `TERM_GRACE_S` lowered: `b` cancelled (`junit.xml`'s
     `test abort`, SIGTERM's message), `c` in `unobserved`, `results.json`'s
     `cancelled` `"global-timeout"`, `run.json`'s `starved` true; the run ends within
     a few seconds.
   - *A starved pass:* a test that writes `skein-world: starved: waited 1 ms runnable,
     ...` with `eprintln!` and unwinds with `std::panic::resume_unwind`, then another
     test: the first is `starved`, its output holding the line, not rerun; the second
     passed in the same process (one process). A test that writes the line and then
     panics with a message is `failed`.
   - *A failure that passes alone:* `a_sets` sets a static, `b_reads` fails when it is
     set: `b_reads` failed, `before` `["a_sets"]`, `alone` `"passed"`, and
     `command-output.txt` holds `may depend on an earlier test` beside its name and
     `a_sets`. A failure first in its process has `alone` null and no alone process.
   - *A split binary:* `LANES = 2`; kept times in the fixture's
     `TARGET/tiers-one-process/times.json` making one binary of four tests longer than
     the total over the lanes: two processes for it, each two contiguous tests in
     name order. With no kept times, by test count. After the run, `times.json` holds
     this run's tests and keeps an entry of a test it did not run.
   - *Groups and priorities:* `LANES = 2`, two binaries in a test group with
     `max-threads = 1`: their processes' intervals never overlap; `LANES = 1`, a
     binary with `priority = 10` starts before a longer one with none. A test in the
     group `process-of-its-own` runs in a process of its own.
   - *The environment:* a fixture test that asserts, and so passes only when it holds:
     `TIERS_ONE_PROCESS=1`, `NEXTEST_PROFILE`, no `NEXTEST` though the caller's `env`
     has `NEXTEST=1`, `CARGO_MANIFEST_DIR` its `cwd`, `CARGO_PKG_NAME`, a
     `CARGO_BIN_EXE_<name>`, `OUT_DIR`, the library path's `deps` entry; its working
     directory the listing's `cwd`.
   - *The wall hard limit:* `limits = {PROFILE: {'wall_hard_s': 0.5}}` with a hanging
     test, the graces lowered: `starved.json`, `run.json`'s `starved`, `cancelled`
     `"wall-hard"`, no process left in the session.
   - *A leak:* a test that leaves a child holding its stdout: the process is `leaked`,
     and the run goes on to the next process within `LEAK_S` and a little.
   - *An empty selection:* `selected` 0, `no_tests` `"fail"` (and `"pass"` with
     `--no-tests=pass`), `run.json`'s `returncode` null, no lanes run.
   - *Refusals*, a table, each a `ValueError` before the fake cargo or the prefix
     runs: `test-threads` in the profile, `-j 2`, a setup script, `retries = 1`,
     `slow-timeout = { period = "1s", terminate-after = 2 }`, an override with
     `threads-required`, `--no-run`, `--config-file`, another `--profile`, a
     `cpu_test_s` limit, an override filter with `rdeps(x)`, a directory that holds a
     file.
   - *Filtersets*, a table of expression, test and result over synthetic tests:
     each predicate and matcher with its default, `not`/`!`, `and`/`&`, `or`/`|`/`+`,
     `-`, parentheses and precedence; the profile's `default-filter` disagreeing with
     the listing's `filter-match` refuses the run, and a caller's `-E` skips that
     check.
3. **`one_process.py`**: the refusals and profile reading; the listing and selection;
   the filtersets and overrides; the environment; `plan.json`; the lanes (estimates,
   splitting, groups, priorities, spawning, two pipes per process, announcements and
   log lines, outcomes, fresh and alone processes, `wait4`, leak, global timeout,
   SIGTERM from `run`); the events; `run`'s supervision (session, wall hard limit,
   interruption, as `execute`'s), status lines as they come, and the files, then
   `times.json`.
4. The real package (Acceptance, fourth item), from the slot.
5. `python3 -I -B tools/tiers/test_one_process.py` (through heavy, `notes/common.md`)
   and `python3 -I -B tools/tiers/test_cpu.py`, unchanged and green beside it;
   `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As the implementer's text gives it, with: the libtest lines the fixtures showed (an
announcement, a log line of each result); the rule each row broke and the failure
seen; the `skein-lib` counts; the suite's wall and its fixture build CPU.
