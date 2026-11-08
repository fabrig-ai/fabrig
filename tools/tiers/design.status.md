# design: status

2026-11-08. Where the slow tier runner's code lags or departs from [design.md](design.md).

Section 7.3 has no known gap in Temper's fuzzy gate entry: `cpu_suite_s`
alone at 120, after Jig's entry, with the requested comment after Jig's
and no `wall_hard_s`. The gate figures beside it agree with the user's
word recorded in `budgets.toml`. Section 8 keeps the unset focused entries
and gate-share measurement outside the runner deferred.

- **Not yet:** 4: a `df` that fails in `run --cpu`'s last disk wait, the one before nextest, is no failed disk check: `subprocess.CalledProcessError` passes `main`'s handler, so the run ends in a traceback and exit 1, not 2. `step` and `list_tests` catch it. 7.5: the same `df` before `cpu-run`'s build passes that launcher's handler too, with exit 1 in place of a failed build's 2. The request for plain commands names neither, so its plan left both.
- **Departs:** 10.2, 11.1 and 11.2: `run` reads its `config` argument for
  the profile and overrides, but gives the listing no configuration path.
  A configuration outside the checkout's `.config/nextest.toml` can therefore
  differ from the one nextest selects with. No reason is recorded.
- **Departs:** 11.1 and 11.2: `configuration` compiles `default-filter`
  even with a caller's filter, so a default filter outside the supported
  subset refuses the run even when its agreement check would be skipped.
  No reason is recorded.
- **Departs:** 11.2: a joined `-EEXPR` or `--filterset=EXPR` is forwarded to
  the listing but `arguments` does not recognise it as a caller's filter. The
  default-filter agreement check can refuse that selection. No reason is
  recorded.
- **Departs:** 11.2: an empty selection with `--no-tests=warn` returns 0
  without the warning. No reason is recorded.
- **Departs:** 11.4: fresh processes after a crash are prepended to waiting
  jobs without ordering them by their remaining tests' priority and
  estimate. No reason is recorded.
- **Departs:** 11.6 and 11.7: `BinaryProcess.completed` uses libtest's
  result even after the process has been marked cancelled. A test that
  handles SIGTERM and reports a result can be passed or failed, and later
  tests in that process can still be reported, instead of the running test
  being cancelled and the rest unobserved. No reason is recorded.
- **Departs:** 11.7: cancellation leaves waiting jobs in `lanes`' queue,
  and its loop continues while that queue is nonempty even after every
  running process has been reaped. Such a global timeout does not finish;
  a wall hard cancellation eventually kills the lanes and `run` refuses
  their nonzero exit instead of completing a starved run. No reason is
  recorded.
- **Departs:** 10.2, 11.7 and 11.8: if a forced session kill leaves a
  started process without its final event, `run` raises `ValueError`
  before writing the finished results, rather than returning a starved
  result. Its comment gives the reason: do not invent rusage or mark an
  incomplete result finished.
- **Departs:** 11.8: `command-output.txt` has the original failure's output
  and a passing-alone label, but no diagnostic output from the alone
  process and no line for a failed-alone result. The alone result is kept
  in `results.json`. No reason is recorded.
- **Departs:** 11.8: a cancelled test whose process handles the signal and
  exits with a code instead of a signal gets a JUnit `test failure` with
  `test failed`, not the specified cancellation `test abort` and signal
  message. No reason is recorded.
- **Design departs:** from the user's request for plain commands, and the code with it. The plan's cut decided each; no signature of section 10 changes.
  - 7.3: SIGKILL goes to every process left in the command's session, not to its process group alone, because nextest gives each test a group of its own, and a signal to the command's group would leave tests running once nextest is dead. SIGTERM goes to the group, as asked.
  - 7.3, 10.2: in a starved run, an attempt with a record and a JUnit abort by SIGTERM or SIGKILL is starved, not failed, where the request names only a pending attempt and an abort with no record, because SIGTERM first makes such an abort the usual end of an overrun, and failing it would return 100 for a run that only starved. Its cost: a test that died of either signal by itself in a starved run reads as starved.
  - 10.3: `run.json` keeps `cancel_code`, always null, though the request's bound lets only the output file's name change, because cancellation is signals, with no exit status of its own, and section 10 keeps its keys.
  - 7, 10: from section 7 on the setting is named `TIERS_WRAPPER`, and section 10 speaks of the caller's build command, where the request has sections 1 to 3, 5 to 7 and 10 say "the wrapper", because from section 7 on a bare "wrapper" is the CPU wrapper (section 3).
- **Design departs:** from the inproc-fixes request (`temper/docs/plan/inproc-fixes/inputs/request.md`, 1), decided by its cut. 11.7: a process a forced kill left without its record keeps the outcomes libtest had reported (passed, failed, starved), with no process, where the request has that process's tests unobserved, because a failure reported before the kill stays red, as 7.3 keeps an attempt that has its record; its running test, the tests after it and a crashed or cancelled one are unobserved, as asked.
