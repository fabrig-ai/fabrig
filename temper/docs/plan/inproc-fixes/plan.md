# inproc-fixes: the plan

Every one-process departure in `tools/tiers/design.status.md` closes before the gate
runs focused and fuzzy through the mode (`inputs/request.md`): the code keeps to
`tools/tiers/design.md`, 11, or 11 changes where the code's choice is better, saying
why. Nextest's path, 10.2's contract, budgets, timeouts, filters and tiers stay.

Contracts: `notes/contracts.md`; standing rules: `notes/common.md`.

## Parts and tasks

1. `runner` (tools/tiers): `one_process.py` to 11 as revised: cancellation drops the
   waiting processes and finishes; a forced kill gives a starved result; a cancelled
   test stays cancelled whatever libtest reports after, and its JUnit is a
   `test abort`; the listing gets `--config-file`; every spelling of a caller's filter
   counts, and the default filter is compiled only when used; `--no-tests=warn` warns;
   a fresh process takes its place by priority and estimate; the alone diagnostic's
   line and output reach `command-output.txt`. A row each in `test_one_process.py`,
   the status entries, and at the end the two python suites and the focused and fuzzy
   suites through `tools/tiers/one-process`. Needs nothing.
2. `summary` (tools/tiers): `cpu.py`'s `one_process_summary` reads a test whose
   `process` is `null` after a forced kill (`notes/contracts.md`, 1), refusing it
   elsewhere. Needs nothing.

Two tasks, as the request allows (one or two), both at effort medium, released
together, in parallel: their files are disjoint (`one_process.py` and its tests;
`cpu.py` and its tests), and they meet at `results.json`'s one new value, fixed in
`notes/contracts.md`, 1. The critical path is `runner` alone, its end runs of the
whole workspace's focused and fuzzy suites the longest step.

## Decisions

- **The design is written by this cut:** `tools/tiers/design.md`, 11.1 (the default
  filter's refusal only where used; a stale "11.7's files" now 11.8's), 11.2 (the
  listing's `--config-file`, a caller's filter in every spelling, the default filter
  compiled only when used, the warning's words), 11.4 (the named pipe and why, the
  split capped by the binary's tests and why, the fresh process's place), 11.6 (a
  cancelled test whatever libtest reports after), 11.7 (cancellation drops what waits;
  *A forced kill*), 11.8 (`run.json` after a forced kill, the alone lines in
  `command-output.txt`, a cancelled test's JUnit, `results.json`'s test without a
  process, the named pipes outside the run directory) and 11.10.
- **Where 11 took the code's side:** the named pipe (11.4, 11.8): libtest opens its
  logfile close-on-exec, so an inherited `/dev/fd/N` would reach a test's children
  against their descriptor isolation; and the split's cap by the binary's test count
  (11.4): a process given no names would run every test of its binary. Both status
  entries went with this cut.
- **A forced kill's reported tests keep their outcomes** (11.7). The request has "that
  process's tests unobserved"; the cut keeps what libtest reported before the kill
  (passed, failed, starved), with no process, so that a failure stays red, as 7.3
  keeps an attempt that has its record; the running test, those after it and a crashed
  or cancelled one are unobserved. This is a **Design departs** entry in
  `tools/tiers/design.status.md`. Its CPU is unknown and in no total: the suite's CPU
  is a lower bound, with no rusage invented. 10.2's text is unchanged: its
  `processes` and suite sums are over the processes with a record.
- **The warning's words** are nextest's, `warning: no tests to run`.
- **No survey:** the sites are two files the cut read whole; their readers are
  `test_one_process.py`, `test_cpu.py` and `cpu.py`'s `execute_one_process`, named in
  the specs.

## Overlaps with drafts beside this one

None: no other draft in flight names a file under `tools/tiers/` or this plan's
directory.
