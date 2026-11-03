+++
id = "unobserved-aborts"
part = "unobserved"
kind = "feature"
needs = []
files = [
  "tools/tiers/cpu.py",
  "tools/tiers/test_cpu.py",
  "tools/tiers/design.status.md",
]
effort = "high"
minutes = 50
+++

# A cancellation abort with no record is unobserved in a starved run

## Goal

A starved run's JUnit attempt with no record, aborted by SIGTERM or SIGKILL (nextest's
cancellation), makes `cpu.py` `summary` raise `record coverage: missing [...]` (exit 2):
nextest killed its wrapper before the pre-exec record. Make such an attempt **unobserved**
(`tools/tiers/design.md`, 7.3): no coverage error, no failure, no executed attempt; the
summary lists it, counts it and warns of it, green. Every other missing record stays exit 2.

Acceptance:

- In a run whose `run.json` has `starved: true`, a JUnit attempt with no record whose
  outcome element's `type` contains `abort` and whose `message` attribute or text names
  `aborted with signal 15` or `aborted with signal 9` is unobserved: `summary` returns,
  `result['unobserved']` lists it as `{'binary_id': ..., 'test_name': ..., 'attempt': ...}`
  (sorted by identity), `result['suite']['unobserved']` counts it, `result['warnings']`
  holds one line for it (text below), and it is in neither `failures`, `tests`,
  `suite.attempts` nor `suite.tests`. The verdict is `starved` (unless over budget), and
  `cpu-summary` exits 0.
- The same attempt in a run that is not starved still fails coverage: `cpu-summary` exits
  2 with `record coverage: missing [('qa-jig', 'fixture::suite', 'two', 1)]`.
- In a starved run, a missing record stays exit 2 when JUnit shows the attempt passed,
  failed by assertion or exit code, aborted by any other signal (11), or timed out.
- An earlier observed failure of the same test stays in `failures` when a later attempt
  is unobserved; unexpected records keep today's rule (accepted only in a starved run).
- `execute` in a starved run whose JUnit holds such an abort returns 0.
- Every summary, the tools summary included, has a top-level `unobserved` list and a
  `suite.unobserved` count (`[]` and 0 when there is none); no existing key changes name,
  type or meaning.
- `heavy python3 -I -B tools/tiers/test_cpu.py` and `heavy python3 -I -B
  tools/tiers/test_run.py` pass; `tools/tiers/design.status.md` says no gap.

## Inputs

- `temper/docs/plan/collector-starved/notes/common.md`.
- `tools/tiers/design.md`, 7.3, the paragraph "The summary verifies exactly one record for
  every JUnit attempt" (the sentences from "In a starved run" to "fails coverage (exit 2)"
  are this task's), and "Starvation alone, unobserved attempts included, returns 0"; 7.4,
  "cancellation aborts without records, starved or not".
- `temper/docs/plan/collector-starved/inputs/request.md`: the incident (four
  `temper-forge-setup` tests, `process aborted with signal 15 (SIGTERM)`, no records).
- nextest's JUnit form for an abort (`nextest-runner/src/reporter/aggregator/junit.rs`,
  0.9.137 and 0.9.148 alike): `<failure type="test abort"
  message="process aborted with signal 15 (SIGTERM)">process aborted with signal 15
  (SIGTERM)</failure>`; with leaked handles the type is `test abort (leaked handles)`; a
  retry's abort is a `rerunFailure` (or `flakyFailure`) of the same type and message; with
  several errors the message is `N errors occurred executing test` and the text lists
  `* process aborted with signal 15 (SIGTERM)` among them. SIGKILL is signal 9.

## Owns and may change

Owns, in `tools/tiers/`:

- `cpu.py`: `junit`, `summary`, `summarize_records` and `print_summary`, and one new
  module-level pattern. `records`, `tools_summary`, `execute`, `prepare`, `budgets` and
  `save` stay as they are, and so do the signatures of `summary(directory, limits=None,
  pair=None, expected=None)` and `summarize_records(items, pair, limits, run, timeouts,
  failures)`: fab replaces `summarize_records` by name with a six-parameter function
  (common note). `junit` is read only by `summary`; its return shape is yours.
- `test_cpu.py`: the new tests below in `CollectorTests`, and a module-level helper for
  the abort element if you want one.
- `design.status.md`: its **Not yet** entry becomes the no-gap text below, nothing else.

Not yours: `tools/tiers/design.md`, `cpu-summary` (its exit stays `over_budget`'s),
`cpu-wrapper.c`, `run`, `cpu-run`, the agents repository.

Pre-approved options:

- If an existing test in `test_cpu.py` or `test_run.py` asserts the old suite line or the
  summary's exact keys, update it for the added `unobserved` only, and list it in the
  report.
- If you keep `junit`'s `failures` a list and filter elsewhere, that is yours, so long as
  a test fails only through attempts that are not unobserved.
- If a JUnit abort element has neither `message` nor text naming the signal, it is not
  unobserved (stays exit 2); do not infer the signal from anything else.

## Steps

1. `cpu.py`. One way to shape it (the inside is yours):

   ```python
   # nextest's abort message for the signals of cancellation (tools/tiers/design.md, 7.3).
   CANCELLED = re.compile(r'\baborted with signal (?:9|15)\b')

   def junit(directory, profile):
       # returns expected, timeouts, failed, passed, aborts, where
       #   failed: {"BINARY NAME": {its attempts' keys whose outcome is a non-timeout
       #           failure}} for the tests `failures` holds today (a primary element and
       #           a non-timeout outcome);
       #   aborts: the keys whose outcome element's type contains "abort" (any case) and
       #           whose message attribute, else text, matches CANCELLED.

   def summary(directory, limits=None, pair=None, expected=None):
       ...
       unobserved = set()
       if (directory / "junit.xml").exists():
           expected, timeouts, failed, passed, aborts = junit(directory, profile)
           missing = expected - actual
           if run["starved"]:
               # Cancellation can stop a wrapper before its pre-exec record.
               unobserved = missing & aborts
               missing -= unobserved
           extra = actual - expected
           if missing or (extra and not run["starved"]):
               raise ValueError(...)  # today's message
           failures = [name for name, keys in failed.items() if keys - unobserved]
       ...
       result = summarize_records(items, pair, limits, run, timeouts, failures)
       # set result['unobserved'], result['suite']['unobserved'] and append the warnings
       return result
   ```

   Check both the `message` attribute and the text, since either may hold the signal;
   the `\b` keeps `signal 90` or `signal 150` out. `summarize_records` adds
   `unobserved=[]` beside `failures` and `unobserved=0` in `suite` (beside `starved_run`),
   so every summary has the keys; `summary` fills them after `summarize_records` returns,
   which keeps fab's replacement working unchanged. Each warning, verbatim:

   ```python
   f"{pair} {binary_id} {test_name}: attempt {attempt} unobserved, cancelled before its record"
   ```

   `print_summary`'s suite line gains `unobserved N` after `starved_run X`:
   `...; starved {starved} starved_run {starved_run} unobserved {unobserved} leaked
   {leaked} incomplete {incomplete}; {verdict}`. Its warnings print as now.

2. `test_cpu.py`, in `CollectorTests` (`setUp` gives profile `qa-jig`, binary
   `fixture::suite`, test `one`; `self.starved(junit=True)` keeps `junit.xml`):

   ```python
   def abort(signal_number, name, kind='test abort', *, message=True):
       text = f'process aborted with signal {signal_number} ({name})'
       attribute = f' message="{text}"' if message else ''
       return f'<failure type="{kind}"{attribute}>{text}</failure>'

   def test_starved_cancellation_abort_without_record_is_unobserved(self):
       self.starved(junit=True)
       self.record()
       for element in (abort(15, 'SIGTERM'), abort(9, 'SIGKILL'),
                       abort(15, 'SIGTERM', 'test abort (leaked handles)'),
                       abort(15, 'SIGTERM', message=False)):
           with self.subTest(element=element):
               self.xml(f'<testcase name="one"/><testcase name="two">{element}</testcase>')
               result = CPU['summary'](self.directory)
               # unobserved == [{'binary_id': 'fixture::suite', 'test_name': 'two', 'attempt': 1}]
               # suite.unobserved 1; failures []; tests only 'one'; suite attempts 1, tests 1;
               # verdict 'starved'; the warning line, verbatim
               # self.cli() exits 0, its stdout holds 'unobserved 1' and
               # 'warning: qa-jig fixture::suite two: attempt 1 unobserved, cancelled before its record'
       self.assert_cancelled_verdict(0)   # execute, starved by the hard wall, JUnit kept

   def test_cancellation_abort_without_record_fails_coverage_when_not_starved(self):
       self.record()
       self.xml(f'<testcase name="one"/><testcase name="two">{abort(15, "SIGTERM")}</testcase>')
       # self.cli() exits 2; stderr holds
       # "record coverage: missing [('qa-jig', 'fixture::suite', 'two', 1)]"

   def test_starved_run_missing_other_records_fails_coverage(self):
       self.starved(junit=True)
       self.record()
       for case in ('<testcase name="two"/>',
                    '<testcase name="two"><failure type="test failure with exit code 101" '
                    'message="thread \'two\' panicked"/></testcase>',
                    f'<testcase name="two">{abort(11, "SIGSEGV")}</testcase>',
                    '<testcase name="two"><failure type="test timeout"/></testcase>'):
           # in a subTest: self.xml('<testcase name="one"/>' + case); self.cli() exits 2
           # with 'record coverage: missing' in stderr

   def test_unobserved_retry_keeps_the_observed_failure(self):
       self.starved(junit=True)
       self.record(exit_code=7)
       self.xml('<testcase name="one"><failure type="test failure with exit code 7"/>'
                '<rerunFailure type="test abort" '
                'message="process aborted with signal 15 (SIGTERM)"/></testcase>')
       # unobserved == [{'binary_id': 'fixture::suite', 'test_name': 'one', 'attempt': 2}]
       # failures == ['fixture::suite one']
   ```

   Also assert, in an existing tools test (`test_command_summary_tools_budgets_and_multiple_suites`
   or the nearest), that a tools summary has `unobserved == []` and `suite.unobserved == 0`.

3. Run the new tests alone, then the suites, one command at a time, reading each count
   (`Ran N tests` and `OK`; a run that ran none proves nothing):

   ```sh
   heavy python3 -I -B tools/tiers/test_cpu.py CollectorTests.test_starved_cancellation_abort_without_record_is_unobserved CollectorTests.test_cancellation_abort_without_record_fails_coverage_when_not_starved CollectorTests.test_starved_run_missing_other_records_fails_coverage CollectorTests.test_unobserved_retry_keeps_the_observed_failure
   heavy python3 -I -B tools/tiers/test_cpu.py
   heavy python3 -I -B tools/tiers/test_run.py
   ```

   `summary` now accepts what it refused, so its readers' tests run whole: both suites
   above (`cpu-summary`, `execute` and `run --cpu` are in them). fab's own tests are the
   agents repository's, not runnable here; the report says so.

4. `tools/tiers/design.status.md`: replace the date with today's (`date -u +%F`) and the
   **Not yet** entry with the line `No gap.`, so the file is its title, the dated line
   and `No gap.`. Then `python3 -I -B tools/citations/citations.py --base origin/main`.

5. Commit (the collector and its tests in one commit, the status in another, each saying
   why), push your branch.

## Report

As the implementer's text says, and also: the shape chosen for `junit`'s return; the test
counts of both suites; and, under `## Left`, that fab's `cases()` (the agents repository's
`fab/cpu.py`) still turns the unobserved attempt's raw JUnit `test abort` into a `fail`
case, so `queue_cpu.summarize` counts it failed and the gate step stays red until fab maps
the summary's `unobserved` entries to its `skipped` (unobserved) result; not this
repository's.
