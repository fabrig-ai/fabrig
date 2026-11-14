# design: status

2026-11-14. Where the slow tier runner's code lags or departs from [design.md](design.md).

- **Not yet:** 10.2, 11.5, 11.6, 11.10: a starved pass in a one-process run unwinds (`skein_world::pass_starved`), libtest reports it failed, and the lanes read it starved by its last line whatever libtest's result, so a failure whose output ends with the line reads starved; a process a starved pass ends with exit 0 reads as a crash, and only a crash starts a fresh process; the fixture's starved case unwinds with `resume_unwind`.
- **Not yet:** 3, 11.9, 11.10: no entry point reads `TIERS_HOLD`: one-process by hand takes no hold, so the executables its listing built run after cargo has exited with nothing holding their build directory, and its suite has no fake hold [in flight: change 1023].

Section 7.3 has no known gap in the gate entries: Skein's `default` is
`cpu_suite_s` alone at 40, with the header's Skein line naming the user's
word of 11-08 for it and of 11-07 for the fuzzy 120; Temper's fuzzy entry
is `cpu_suite_s` alone at 180, by the user's word of 11-08, after Jig's
entry, with its comment after Jig's and no `wall_hard_s`. The other gate figures agree
with the user's word recorded in `budgets.toml`. Section 8 keeps the unset focused entries,
gate-share measurement outside the runner and the revisit of the four
figures raised by half on 11-08 deferred.

Sections 10 and 11 have the one-process runner, summary and hand command:
process CPU from `wait4`, null per-test CPU, selected-test results and
JUnit, fresh processes after crashes, and alone diagnostics that keep
failures red. Sections 11.1 and 11.2 select with the caller's configuration
and filter spellings, compile the default filter only when used, and warn
on an empty selection as requested. Sections 11.4 and 11.8 use the named
pipe with libtest's close-on-exec writer, order fresh jobs by priority and
estimate, and forward alone diagnostics' lines and output.

Sections 11.6 to 11.8 drop waiting jobs on cancellation, preserve results
read before the cancellation mark, keep the running test cancelled despite
later libtest results, and give it a JUnit abort even after a normal exit.
A forced kill finishes starved: reported outcomes survive without their
process record, pending outcomes are unobserved, and unknown process CPU
is excluded. The summary accepts those recordless outcomes only with
cancellation and keeps a reported failure red. The fixture rows cover
these paths, including process-index remapping. No implementation gap is
known in these fixes.

The draft departures below remain: the inproc-fixes cut explicitly kept
the older scheduling and diagnostic choices, whose reasons are still
unrecorded, and chose to retain outcomes reported before a forced kill.
Section 8 carries the two unresolved choices, caller adoption and CPU
measurement. No `process-of-its-own` override is configured.

- **Not yet:** 4: a `df` that fails in `run --cpu`'s last disk wait, the one before nextest, is no failed disk check: `subprocess.CalledProcessError` passes `main`'s handler, so the run ends in a traceback and exit 1, not 2. `step` and `list_tests` catch it. 7.5: the same `df` before `cpu-run`'s build passes that launcher's handler too, with exit 1 in place of a failed build's 2. The request for plain commands names neither, so its plan left both.
- **Design departs:** 11.4 and 11.8: the inproc-runner request asks for
  ordering by previous process times, falling back to test counts. The
  design and code instead keep per-test wall times, sum them for each
  job and estimate an unmeasured test from the selection's mean. The
  plan's cut specifies this choice but records no reason.
- **Design departs:** 11.6: the inproc-runner request asks for every
  failure to run once more alone. The design and code rerun only failed
  or crashed tests with earlier tests in their process; a first-test
  failure has no alone diagnostic. The cut records no reason for this
  narrower diagnostic.
- **Design departs:** from the user's request for plain commands, and the code with it. The plan's cut decided each; no signature of section 10 changes.
  - 7.3: SIGKILL goes to every process left in the command's session, not to its process group alone, because nextest gives each test a group of its own, and a signal to the command's group would leave tests running once nextest is dead. SIGTERM goes to the group, as asked.
  - 7.3, 10.2: in a starved run, an attempt with a record and a JUnit abort by SIGTERM or SIGKILL is starved, not failed, where the request names only a pending attempt and an abort with no record, because SIGTERM first makes such an abort the usual end of an overrun, and failing it would return 100 for a run that only starved. Its cost: a test that died of either signal by itself in a starved run reads as starved.
  - 10.3: `run.json` keeps `cancel_code`, always null, though the request's bound lets only the output file's name change, because cancellation is signals, with no exit status of its own, and section 10 keeps its keys.
  - 7, 10: from section 7 on the setting is named `TIERS_WRAPPER`, and section 10 speaks of the caller's build command, where the request has sections 1 to 3, 5 to 7 and 10 say "the wrapper", because from section 7 on a bare "wrapper" is the CPU wrapper (section 3).
- **Design departs:** from the inproc-fixes request, decided by its cut. 11.7: a process a forced kill left without its record keeps the outcomes libtest had reported (passed, failed, starved), with no process, where the request has that process's tests unobserved, because a failure reported before the kill stays red, as 7.3 keeps an attempt that has its record; its running test, the tests after it and a crashed or cancelled one are unobserved, as asked.
