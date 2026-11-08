# ignored-misc: rules every task keeps

- **The request** is `temper/docs/plan/ignored-misc/inputs/request.md`:
  six of temper's ignored tests each name a gap in the product, and
  this plan fixes each gap at its cause, by the design its reason cites,
  so the tests run again. Read it for the why. Build what your spec and
  the design sections it names say.
- **Each test your spec names loses its `#[ignore]` and passes in the
  suite that selects it.** A test whose design has moved is rewritten to
  the design, never deleted. If its gap turns out to be a feature larger
  than your task, the test keeps its `#[ignore]`, its reason names the
  missing design section, and your report says so first, with what you
  found; your spec says where that option applies.
- **Only focused and fuzzy run,** with one exception: a process story
  your spec names, by name and alone, to see it pass, as this repairs a
  finding:
  `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=NAME)'`,
  count 1. Nothing else of `process-temper` runs, and never
  `tools/tiers/run`, a browser or live tier, `scripts/browser.sh`, a
  walk, a render check or a throwaway driver.
- **A focused test that drops its `#[ignore]` runs alone first**
  (`heavy cargo nextest run -p PKG --test BIN -E 'test(=PATH)'`, count 1,
  `PATH` the test's full name with its modules, as `nextest list`
  shows it), then with its package. Report its time from nextest. A
  test over its guide (`skein/docs/foundation/testing-strategy.md`, 8)
  is examined for its cause (8.1, question 2) and named in the report;
  it is never moved or ignored again for its cost.
- **No budget, timeout or tier changes:** no figure in
  `tools/tiers/budgets.toml`, and no timeout, slow period, thread count
  or filter in `.config/nextest.toml`. No other ignored test is touched.
- **A change of meaning runs its readers' tests:** where your fix
  changes what a shared item means (a value it returns, what a restart
  keeps, when a host is placeable), the focused tests of every package
  that reads it run too; your spec names them.
- **Old forms:** `python3 -I -B <tree>/scripts/old_forms.py` once before
  pushing, for each tree you touch. A raised count is yours to fix.
- **Design and status lines:** the design is written (`## Inputs` names
  it). A task edits only the status entries its spec names, in the words
  the spec gives. That departs from the implementer's text, which closes
  `docs/design/` to you: those entries are your task's.
- **Decide, don't ask.** A small edit just outside your file list is
  made, and named in the report. A `Question:` is only for what truly
  blocks your acceptance.
- **Before calling a focused or fuzzy failure "outside the change,"** run
  it once on the pristine base. If it passes there and fails on your
  branch, it is your branch's.
- **A new test must bite** (the implementer's text): break the rule once
  by hand, see the test fail for the rule's reason, restore it, and name
  both in the report.
- **Citations:** after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the
  repository root. It reads HEAD, not the working tree.
