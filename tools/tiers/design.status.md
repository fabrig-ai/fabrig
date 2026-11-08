# design: status

2026-11-08. Where the slow tier runner's code lags or departs from [design.md](design.md).

Section 7.3 has no known gap in Temper's fuzzy gate entry: `cpu_suite_s`
alone at 120, after Jig's entry, with the requested comment after Jig's
and no `wall_hard_s`. The gate figures beside it agree with the user's
word recorded in `budgets.toml`. Section 8 keeps the unset focused entries
and gate-share measurement outside the runner deferred.

- **Not yet:** 4: a `df` that fails in `run --cpu`'s last disk wait, the one before nextest, is no failed disk check: `subprocess.CalledProcessError` passes `main`'s handler, so the run ends in a traceback and exit 1, not 2. `step` and `list_tests` catch it. 7.5: the same `df` before `cpu-run`'s build passes that launcher's handler too, with exit 1 in place of a failed build's 2. The request for plain commands names neither, so its plan left both.
- **Design departs:** from the user's request for plain commands, and the code with it. The plan's cut decided each; no signature of section 10 changes.
  - 7.3: SIGKILL goes to every process left in the command's session, not to its process group alone, because nextest gives each test a group of its own, and a signal to the command's group would leave tests running once nextest is dead. SIGTERM goes to the group, as asked.
  - 7.3, 10.2: in a starved run, an attempt with a record and a JUnit abort by SIGTERM or SIGKILL is starved, not failed, where the request names only a pending attempt and an abort with no record, because SIGTERM first makes such an abort the usual end of an overrun, and failing it would return 100 for a run that only starved. Its cost: a test that died of either signal by itself in a starved run reads as starved.
  - 10.3: `run.json` keeps `cancel_code`, always null, though the request's bound lets only the output file's name change, because cancellation is signals, with no exit status of its own, and section 10 keeps its keys.
  - 7, 10: from section 7 on the setting is named `TIERS_WRAPPER`, and section 10 speaks of the caller's build command, where the request has sections 1 to 3, 5 to 7 and 10 say "the wrapper", because from section 7 on a bare "wrapper" is the CPU wrapper (section 3).
- **Design departs:** from the inproc-fixes request (`temper/docs/plan/inproc-fixes/inputs/request.md`, 1), decided by its cut. 11.7: a process a forced kill left without its record keeps the outcomes libtest had reported (passed, failed, starved), with no process, where the request has that process's tests unobserved, because a failure reported before the kill stays red, as 7.3 keeps an attempt that has its record; its running test, the tests after it and a crashed or cancelled one are unobserved, as asked.
