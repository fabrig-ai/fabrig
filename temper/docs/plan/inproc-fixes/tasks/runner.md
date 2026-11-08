+++
id = "runner"
part = "runner"
kind = "feature"
needs = []
files = ["tools/tiers/one_process.py", "tools/tiers/test_one_process.py", "tools/tiers/fixtures/one-process/**", "tools/tiers/design.status.md"]
effort = "medium"
minutes = 150
+++

# inproc-fixes: the one-process runner keeps to its design, 11, before the gate uses it

## Goal

`tools/tiers/one_process.py` keeps to `tools/tiers/design.md`, 11 as this plan's cut
revised it: cancellation finishes, a forced kill gives a starved result, a cancelled
test stays cancelled, the listing selects with the caller's configuration and filters,
a fresh process waits its turn and the alone diagnostic shows in the output. Each fix
comes with a row in `tools/tiers/test_one_process.py` that fails before it.

Acceptance:

- Each rule below holds, each with its row, seen failing before its fix (the report
  names the failure seen).
- `tools/tiers/design.status.md`'s **Departs** entries for these sections are deleted,
  each once its fix holds (a stated departure from the implementer's text): the ones
  beginning "10.2, 11.1 and 11.2: `run` reads its `config`", "11.1 and 11.2:
  `configuration` compiles", "11.2: a joined `-EEXPR`", "11.2: an empty selection",
  "11.4: fresh processes", "11.6 and 11.7: `BinaryProcess.completed`", "11.7:
  cancellation leaves waiting jobs", "10.2, 11.7 and 11.8: if a forced session kill",
  "11.8: `command-output.txt`" and "11.8: a cancelled test". Every other entry stays,
  the **Design departs** ones included.
- At the end, once each: `heavy python3 -I -B tools/tiers/test_one_process.py` and
  `heavy python3 -I -B tools/tiers/test_cpu.py`, green; then, from the slot's root, the
  whole workspace each, `TIERS_WRAPPER=heavy tools/tiers/one-process default` and
  `TIERS_WRAPPER=heavy tools/tiers/one-process fuzzy`, green: every failure in them is
  this branch's and repaired, or fails on the pristine base too and goes under
  `## Left` with both logs. A starved or unobserved test is a warning, never red. No CPU
  figure is read or compared.

## Inputs

- `tools/tiers/design.md`, 11.1, 11.2, 11.4, 11.6, 11.7 (with *A forced kill*), 11.8
  and 11.10; 7.3 for starved and unobserved; 10.2's `execute_one_process` for context.
- `temper/docs/plan/inproc-fixes/notes/common.md`.
- `temper/docs/plan/inproc-fixes/notes/contracts.md`, 1: what `results.json` holds
  after a forced kill. Task `summary` makes `cpu.py` read it, beside this task; this
  task's rows read `results.json`, `run.json` and `junit.xml` through `invoke`, never
  `summary`, for a forced kill.
- `temper/docs/plan/inproc-runner/notes/contracts.md`, 2: the rest of `results.json`.
- `tools/tiers/design.status.md`: the entries above say what the code does now.

## Owns and may change

- Owns `tools/tiers/one_process.py`, `tools/tiers/test_one_process.py`, the fixtures in
  `tools/tiers/fixtures/one-process/` (new `FIXTURE_CASE`s in `tests.rs`), and the
  deletions above in `tools/tiers/design.status.md`.
- `cpu.py` and `test_cpu.py` are task `summary`'s: not edited here. If a row of
  `test_cpu.py` fails on this branch, it goes under `## Left`, named.
- Existing rows whose expectations change with a rule are updated here: the listing's
  argument list in `test_passing_binary_and_prefix` (it gains `--config-file`), and any
  other the rules below change, each named in the report.
- `one_process.py`'s docstring cites `temper/docs/plan/inproc-fixes/notes/contracts.md`,
  1 beside the inproc-runner note.
- Internal functions (`arguments`, `configuration`, `make_plan`, `split_jobs`,
  `BinaryProcess`, `lanes`, `write_junit`) may change their signatures; `run`'s
  signature, the `lanes` script's arguments and the event line prefix stay.

## Steps

Each rule: its row first, failing; then the fix.

1. **Cancellation drops what waits (11.7).** In `lanes`, when `cancelled` is first set,
   clear `jobs`: their tests are unobserved, which `run` already reports for any planned
   test with no test event. Mark and `SIGTERM` only the processes not yet reaped
   (`reaped_at is None`); a process reaped before the mark settles as it would have. The
   loop then ends once `active` is empty, with exit 0.
   Row `test_cancellation_drops_waiting_processes`: `LANES=1`, `FIXTURE_CASE=hang`, a
   second binary (`self.binary('fixture::second', names=('d',))`); (a) `global-timeout="1s"`
   with `{'default': {'wall_hard_s': 3}}` as a bound: `cancelled` `global-timeout`,
   `fixture::second d` in `unobserved`, elapsed under 3 s (today: the lanes loop, the
   wall hard kill comes, `run` raises); (b) no global timeout, `wall_hard_s` 0.5:
   `cancelled` `wall-hard`, `d` unobserved, `run.json` `starved` (today: `run` raises on
   the lanes' nonzero exit).
2. **A cancelled test is cancelled (11.6, 11.7).** `BinaryProcess` keeps the mark's
   moment (`cancelled_at`, from `time.monotonic()`, or an equivalent). Once marked: a
   log result read after the mark is not recorded; a test announced after the mark is
   never finished (unobserved); at settlement the test announced before the mark with
   no result recorded before it is `cancelled`. A result read before the mark keeps
   its outcome, even where its output's result word arrives later. The process
   record's `tests` stay the names announced.
   New fixture case `term-pass` in `tests.rs`: `b` installs a `SIGTERM` handler (a
   `static AtomicBool`; `unsafe extern "C" { fn signal(signum: i32, handler: usize) ->
   usize; }`), waits for it, and returns; `c` passes. Row
   `test_cancelled_test_whatever_libtest_reports`: `global-timeout="1s"`, and
   `TERM_GRACE_S` 5 through `self.run.__globals__` so that the process ends by itself
   before any `SIGKILL`; `b` `cancelled`, `c` unobserved, the process `exit_code` 0
   and `signal` `null`; `junit.xml`'s `b` has a
   `failure` of type `test abort` whose message holds `signal 15 (SIGTERM)` (today: `b`
   and `c` passed).
3. **JUnit for a cancelled test that exited with a code (11.8).** In `write_junit`, a
   cancelled test is a `test abort` with the message of its process's signal, or
   `SIGTERM`'s where its process exited with a code. A test with a `null` `process`
   (step 4) reads no process: failed is `test failure`, `test failed`; passed and
   starved as now. Step 2's row checks it.
4. **A forced kill finishes, starved (11.7, *A forced kill*; contracts.md, 1).** In
   `run`, the loop's break at `kill_at` is the forced kill; after `stop`, with it:
   - processes with a `start` event and no `process` event have no record; `processes`
     is the recorded ones, densely, and each test's `process` index is remapped to it;
   - a test whose process has no record keeps its outcome with `process` `null` if it
     is `passed`, `failed` or `starved`; a `crashed` or `cancelled` one is dropped, so
     unobserved;
   - the lanes' nonzero exit is accepted, `returncode` as `Popen` gives it;
   - `run.json`'s `wall_s` runs to the later of the last recorded end and the kill's
     moment (`time.monotonic()` when the loop broke).
   Without the forced kill, a missing final event or a nonzero lanes exit still raises
   `ValueError`. New fixture case `term-ignore`: `a` fails (`panic!`), `b` sets
   `SIGTERM` to ignored (`signal(15, 1)`) and hangs, `c` passes. Row
   `test_forced_kill_finishes_starved`: `TERM_GRACE_S` 5 and `WALL_GRACE_S` 0.5 through
   `self.run.__globals__`, `wall_hard_s` 0.5: `run` returns within 3 s; `results.json`
   has `processes` `[]`, `a` `failed` with `process` `null`, `b` and `c` unobserved,
   `cancelled` `wall-hard`; `run.json` `finished` and `starved` true, `returncode` -9;
   `junit.xml` has `a`'s `test failure` and no case for `b` or `c`; the fixture's session
   is empty after (as `test_wall_hard_and_session_cleanup` checks) (today: `ValueError`).
5. **The listing selects with `config` (11.2).** The listing is
   `[*prefix, cargo, 'nextest', 'list', '--message-format', 'json', '--profile', profile,
   '--config-file', str(Path(config).resolve()), *selection]`. Row
   `test_private_configuration_reaches_the_listing`: `config` a file outside
   `.config/` (`self.root / 'private.toml'`); the recorded listing argv (from
   `prefix-argv.jsonl`) holds `--config-file` and that absolute path.
6. **A caller's filter in every spelling; the default filter compiled only when used
   (11.1, 11.2).** `arguments` counts as the caller's: `-E` and `--filterset` and
   `--filter-expr`, apart (`-E EXPR`, the expression the next word, skipped as a value),
   with `=` (`-E=EXPR`, `--filterset=EXPR`, `--filter-expr=EXPR`) and joined (`-EEXPR`:
   a word beginning `-E` and longer, not `--`), with the existing test-name filters,
   `--ignore-default-filter`, `--partition`, `--skip` and `--exact`. `configuration`
   takes `caller_filter` and compiles `default-filter` only without one (returning
   `None` for it otherwise); `run` calls `arguments` before `configuration`, as now, so a
   refusal still comes before any command. Extend
   `test_default_filter_proof_and_caller_filter`: with `default-filter="none()"`, each of
   `['-E', 'all()']`, `['-Eall()']`, `['-E=all()']`, `['--filterset', 'all()']`,
   `['--filterset=all()']`, `['--filter-expr=all()']` selects 3; with
   `default-filter="rdeps(x)"`, `['-E', 'all()']` selects 3 and no caller filter raises
   `ValueError` before any command (no `prefix-argv.jsonl`).
7. **An empty selection under `--no-tests=warn` warns (11.2).** With `selected == 0`
   and `no_tests == 'warn'`, `run` forwards `warning: no tests to run\n`. Extend
   `test_empty_selection`: the line is in `command-output.txt` for `warn` and absent for
   `fail` and `pass`.
8. **A fresh process takes its place (11.4).** One ordering key, `(-max priority,
   -sum estimate)` of a job's tests, serves `split_jobs`' sort and the fresh process: it
   goes before the first waiting job, after any leading alone diagnostic, whose key
   sorts after its own (equal keys keep it after them). Alone diagnostics still go
   first. Row `test_fresh_process_takes_its_place`: `LANES=1`, `FIXTURE_CASE=crash`,
   `fixture::second` with `d`, overrides `filter="binary_id(=fixture::suite) &
   test(=a)"` `priority=10` and `filter="binary_id(=fixture::second)"` `priority=5`;
   processes by `start_s` are `(fixture::suite, tests)`, `(fixture::suite, alone)`,
   `(fixture::second, tests)`, `(fixture::suite, fresh)` (today the fresh one comes
   third).
9. **The alone diagnostic in the output (11.8).** The lanes' `alone` event gains
   `output`, the diagnostic test's output where it did not pass, else `null`. `run`
   forwards, after the existing `may depend on an earlier test` line where it passed,
   or `{binary_id} {test_name}: failed alone too` where it did not, that output and a
   newline when it is not null. Extend `test_crash_fresh_and_before_first`:
   `command-output.txt` holds `fixture::suite b: failed alone too` and `crash output`
   twice.
10. The status entries (acceptance); the end runs (acceptance), each into an `--out`
    directory outside the worktree's tracked files, read by `cpu-summary DIR` and by
    `grep` in `command-output.txt`, never whole; `python3 -I -B
    tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As the implementer's text gives it, with: each row and the failure it showed before
its fix; the rule broken by hand for each and the failure seen; the two suites' runs
through the mode (tests, passed, failed, starved, unobserved, wall; no CPU); what is
`## Left`, with logs.
