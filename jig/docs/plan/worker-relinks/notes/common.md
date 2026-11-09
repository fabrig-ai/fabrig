# worker-relinks: rules every task keeps

- **The request** is `jig/docs/plan/worker-relinks/inputs/request.md`:
  a deployment at fabrig `2c8b9a3f7d` (one engine, one remote worker over
  a forwarded loopback port, about eight goals' tasks) whose worker
  relinked every two minutes, whose engine then stopped taking it back,
  and which did not stop on SIGTERM. The evidence is outside the
  repository: the request's words are all there is. Read it for the why;
  your spec and the design sections it names are what you build.
- **Failing test first.** Each fix is pinned by a test that fails on the
  trunk for the fault's reason before the fix and passes after it. Commit
  the failing test first, on its own, saying which fault it shows; then
  the fix. The report quotes the failure seen before the fix.
- **Candidates are ruled in or out by test, never by assumption.** A spec
  that names candidates gives each a row; a row that passes on the trunk
  rules its candidate out and stays as a guard. The report says, per
  candidate, ruled in or out and by which row.
- **No figure changes.** Intervals, deadlines, graces, budgets and limits
  stay as configured (`jig/crates/jig-protocol-config/src/profile/*.rs`,
  `temper`'s profiles, every fixture's `limits()`). If a test shows a
  figure itself is wrong, stop and ask (`Question:`), with the
  measurement; never change it.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing
  table test, or of the table your spec makes from one test. A story goes
  in a world below the real loop on a simulated clock. No test reaches a
  real host or a real network; no sleep.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a process or live tier
  story, a walk, a render check or a throwaway driver. A process-tier test
  is written and compiled (`--no-run`), not run; the report says so.
- **Every build and test through `heavy`,** one command at a time, with
  `TEMPER_BUNDLE=none` (the machines' cargo config sets it). `cargo fmt`
  builds nothing and runs directly.
- **Trees:** jig never names temper; skein names neither. A change to
  skein's `io`, `sim` or `world` interface goes to `hearth/core` first:
  none is planned here, and needing one is a `Question:`.
- **Status lines:** the design is written (`## Inputs` names it). A task
  removes only the **Not yet** entries its spec names, its lines alone.
  That departs from the implementer's text, which closes `docs/design/`
  to you: those entries are your task's.
- **Before calling a focused or fuzzy failure "outside the change,"** run
  it once alone on the pristine base. If it passes there and fails on your
  branch, it is your branch's.
- **Old forms:** `python3 -I -B <tree>/scripts/old_forms.py` for each tree
  you touch, once before pushing. A raised count is yours to fix.
- **Citations:** after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the
  repository root. It reads HEAD, not the working tree.
- **Decide, don't ask.** A small edit just outside your file list (a
  compiler-forced arm, a call site's new field) is made and named in the
  report. A `Question:` is only for what truly blocks your acceptance.
