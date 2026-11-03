# collector-starved: the plan

From `inputs/request.md`: `configuring-p1/integration-1`'s end check went red with no failing
code. nextest's 120 s global timeout cancelled the default suite. Four tests were aborted by
SIGTERM before their wrappers wrote a record, so `tools/tiers/cpu.py` `summary` raised
`record coverage: missing` (exit 2). The plan is one direct part with one task, as the
request asks.

## Parts and tasks

- **unobserved** (`tools/tiers/design.md, 7.3`), slice 1.
  - `tasks/unobserved-aborts.md`, high, ~50 minutes, needs nothing. The rule applies in a
    run whose `run.json` says `starved: true`, to a JUnit attempt with no record whose
    outcome is a `test abort` naming signal 15 or 9. Such an attempt is unobserved:
    `summary` lists it in `unobserved` (`binary_id`, `test_name`, `attempt`), counts it in
    `suite.unobserved` and adds a warning. The attempt is not a failure and not an
    executed attempt. Every other missing record stays exit 2. Tests go in
    `tools/tiers/test_cpu.py`'s `CollectorTests`, with its own fakes. The task also sets
    `tools/tiers/design.status.md` to no gap.

## Design written by this cut

- `tools/tiers/design.md`:
  - 7.3: the rule, beside "Unstarted tests are never claimed as executed"; `unobserved` in
    the summary's counts; "Starvation alone, unobserved attempts included, returns 0".
  - 7.4: the suite covers cancellation aborts without records.
- `tools/tiers/design.status.md` (new: the document had none): **Not yet** 7.3 and 7.4,
  until the task lands.

## Critical path

`unobserved-aborts`, then fab's end check on the trunk.

## Overlaps

None with the open branches of the drafts beside this one. Their listed files are jig and
temper only, and none touches `tools/tiers/`. `readiness-eintr` and `forgejo-connect` were
cuts in flight with no files listed, so any overlap with them is unknown.

## Calls made

- **Detecting the abort.** nextest 0.9.137 and 0.9.148 write an abort as `<failure
  type="test abort" message="process aborted with signal N (SIGNAME)">` with the same
  text. The rule requires `abort` in the type and `aborted with signal 15` or `9` (word
  bounded) in the message or the text. Retried attempts' `rerunFailure` and
  `flakyFailure` elements count the same way. No other signal or JUnit form is
  unobserved.
- **Why the key is `run.json` `starved`.** Both collectors write it before `summary`
  reads it: the collector's own `execute` on the wall hard limit, and fab's `read` on
  nextest's global-timeout line.
- **New keys only.** The summary gains a top-level `unobserved` list and a
  `suite.unobserved` count, both present in every summary, tools suites included. No key
  changes. `summarize_records` keeps its six parameters, because fab replaces it by name.
  `summary` fills the new keys after `summarize_records` returns, so fab's replacement
  needs no change.
- **What fab still needs** (agents repository, not this plan's). The collector stops
  raising, but fab's `cpu.cases()` builds cases from raw JUnit plus `measured["tests"]`.
  The unobserved attempt has no entry in `tests`, so its `test abort` stays a `fail`
  case. `queue_cpu.summarize` then counts it in `failed` and `run_steps` reads the step
  red. The precheck and the after-landing tiers go through the same `cases()`. fab needs
  one change: in `cases()`, a JUnit case whose identity is in `measured.get("unobserved",
  [])` and not in `measured["failures"]` becomes `skipped`, with error `run stopped`.
  `skipped` is fab's unobserved result: `queue_cpu` counts it as `unobserved`,
  `gate_report` writes it `<skipped/>`, and `after_landing` reads it as unobserved. The
  task's report lists this under `## Left`.
- **Not in scope.** In a starved run, a test that does have a record still counts as a
  failure when its JUnit shows a SIGTERM `test abort`, if the collector runs alone. fab's
  `read` already reconciles those cases, and the request does not ask for this.
- **Why the status file is new.** `DESIGN.md` 2 asks for a status file beside every
  design document, and the design reaches the trunk before the code. The task's last
  step sets it to `No gap.`.
