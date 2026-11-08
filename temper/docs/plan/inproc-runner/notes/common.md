# inproc-runner: what every task holds to

- **Focused and fuzzy alone (the user, 11-06).** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or live tier or one
  of their stories, a walk, a render check or a throwaway driver. Running the focused
  (`default`) and fuzzy suites through the new one-process mode is running focused and
  fuzzy. A plan is complete when it lands.
- **No budget, timeout, deadline, filter, tier or build setting changes.** Test builds
  stay unoptimized. `tools/tiers/budgets.toml` and the profiles' filters,
  `global-timeout`s and `slow-timeout`s in `.config/nextest.toml` stay as they are; a
  harness's patience stays as it is.
- **Nextest's path stays as it is.** `cpu.py`'s `prepare`, `execute`, the records, the
  wrapper, `cpu-run` and `tools/tiers/run` behave as now; `summary`'s keys for a nextest
  run keep their names and meanings, and only gain keys (`tools/tiers/design.md`,
  10.2).
- **Decide, don't ask.** A stored format or protocol of this plan's may change freely,
  with no compatibility. A small edit just outside a task's file list is made and named
  in the report. A `Question:` is only for what truly blocks the task.
- **The shapes are `temper/docs/plan/inproc-runner/notes/contracts.md`'s.** A name,
  variable, field, key, word or file it gives is spelled as it gives it. One it doesn't
  give is yours, inside your files.
- **tools/tiers is Python 3's standard library, as `cpu.py` is:** run with
  `python3 -I -B`, files loaded with `runpy.run_path` and none importable as a module,
  umask 077, no package installed. Its suites run through `heavy` as every test does
  (`heavy python3 -I -B tools/tiers/test_one_process.py`); if heavy refuses a command
  that isn't cargo, run the suite directly and say so in the report. A suite of
  tools/tiers builds its own fixtures into a temporary directory and never reads
  `target/`, as `test_cpu.py` does (`tools/tiers/design.md`, 7.4).
- **The failing test first** for each rule: write it, run it, see it fail for the rule's
  reason, then build. **A new test must bite** (the implementer's text): break the rule
  once by hand, see it fail, restore; the report names both.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the
  pristine base. If the base passes and your branch fails, it's yours.
- **`docs/design/` and `tools/tiers/design.md` stay closed** but for the **Not yet**
  status entries a spec tells you to delete, as a stated departure from the
  implementer's text: the design is written, and its status must say what is built when
  the branch lands. What the code taught goes in the report.
- **Other drafts beside this one** change none of `tools/tiers/`, `skein/testing/` or
  `skein/tests/`. In temper they own, among others, `temper/crates/temper-service/**`,
  `temper/crates/temper/tests/**`, `temper/tests/real-loop/src/fixtures.rs` and
  `temper/tests/engine/domain/tests/suite/`'s `configuring*` and `core_after_project*`:
  a repair there is the smallest edit that fixes the cause, named in the report.
