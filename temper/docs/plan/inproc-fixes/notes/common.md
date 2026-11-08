# inproc-fixes: what every task holds to

- **Focused and fuzzy alone (the user, 11-06).** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or live tier or one
  of their stories, a walk, a render check or a throwaway driver. Running the focused
  (`default`) and fuzzy suites through `tools/tiers/one-process` is running focused and
  fuzzy. A plan is complete when it lands.
- **No budget, timeout, grace, filter, tier or build setting changes.**
  `tools/tiers/budgets.toml` and `.config/nextest.toml` stay as they are; so do
  `one_process.py`'s `TERM_GRACE_S`, `WALL_GRACE_S` and `LEAK_S` (a test may set them
  through the module's globals, as `test_one_process.py` does).
- **Nextest's path and 10.2's contract stay as they are.** `cpu.py`'s `prepare`,
  `execute`, the records, the wrapper, `cpu-run` and `tools/tiers/run` behave as now;
  `summary`'s keys keep their names and meanings (`tools/tiers/design.md`, 10.2).
- **The shapes are `temper/docs/plan/inproc-fixes/notes/contracts.md`'s,** and where it
  says nothing, `temper/docs/plan/inproc-runner/notes/contracts.md`'s. A word, line or
  key `tools/tiers/design.md`, 11 spells out (`warning: no tests to run`,
  `failed alone too`) is spelled as it gives it.
- **tools/tiers is Python 3's standard library, as `cpu.py` is:** run with
  `python3 -I -B`, files loaded with `runpy.run_path` and none importable as a module,
  umask 077, no package installed. Its suites run through `heavy` as every test does
  (`heavy python3 -I -B tools/tiers/test_one_process.py`); if heavy refuses a command
  that isn't cargo, run the suite directly and say so in the report. Each suite builds
  its own fixtures into a temporary directory and never reads `target/`
  (`tools/tiers/design.md`, 11.10).
- **The failing row first,** for each fix: write it, run it, see it fail for the
  rule's reason (the request: "each code fix comes with a row that fails before it"),
  then fix. A row that would hang before its fix carries its own bound (a
  `wall_hard_s` or an elapsed-time assertion behind one), so that it fails rather than
  hangs. The report names each row and the failure it showed before the fix.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the
  pristine base. If the base passes and your branch fails, it's yours.
- **Read and compare no CPU figure** (the request's limit): a run's CPU lines are
  neither checked nor reported.
- **`tools/tiers/design.md` stays closed;** what the code taught goes in the report.
  `tools/tiers/design.status.md` is edited only to delete the entries a spec names, as
  a stated departure from the implementer's text: the design is written, and its
  status must say what is built when the branch lands.
- **Decide, don't ask.** A small edit just outside a task's file list is made and
  named in the report. A `Question:` is only for what truly blocks the task.
- **Other drafts beside this one** change nothing under `tools/tiers/`.
