# ui-away-all: rules every task keeps

- **The request** is `temper/docs/plan/ui-away-all/inputs/request.md`:
  Away on All projects, as `temper/docs/design/draft/surfaces.md`,
  section 2, has it. Read it for the why; your spec and the design
  sections it names are what you build.
- **A plan is complete when it lands (the user, 11-06).** Walks, render
  checks, screenshots and throwaway browser drivers follow landing,
  outside the plan: don't ask for them, or run them.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or
  live tier story, or a tier's CPU figure. This overrides anything in a
  spec.
- **Tests stay cheap:** a new case of one behaviour is a row of an
  existing table test, not a new test (skein's
  `docs/foundation/testing-strategy.md`, 8). temper's focused and fuzzy
  suites go to 30 and 120 CPU seconds in the gate. Budgets and timeouts
  are the user's, never raised.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none`
  (the machines' cargo config sets it). `cargo fmt` builds nothing and
  runs directly.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`** conflict
  across plans: on a catch-up, regenerate rather than hand-merge, and keep
  `names.txt` sorted as it is. Generated files your change makes stale
  (`docs/help/reference/*`, prints, goldens, counted baselines) are yours
  to regenerate by their own tool in the same task.
- **`temper-client-journeys again` is generation, not a walk:** a task
  whose change makes a journey recording stale runs it. A re-record that
  stops on "a key mark caught a moving page" is retried up to three
  times, each on a fresh fake box.
- **`temper.css`:** new rules go under this plan's marker comment, `/*
  ==== ui: AREA ==== */`, never at the end of the file; replaced rules
  are removed where they stand. This plan expects none.
- **A test's oracle or helper that your change's correct behaviour trips
  is yours to repair in the same task,** keeping what it checks for
  everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base passes and the branch
  fails, it's the branch's.
- **Decide, don't ask.** The user is asleep and takes temper/ui's
  judgement. A small edit just outside the file list (a re-export, a
  `mod` line, a golden) is made and named in the report. A `Question:` is
  only for what truly blocks the task.
