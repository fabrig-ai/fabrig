# ui-composer-focus: for every task

The request: `temper/docs/plan/ui-composer-focus/inputs/request.md`, whose *For every task of this plan* binds. Its design is
written and on the trunk: the sections the spec cites.

- **Decide, don't ask.** The user is asleep and takes this plan's judgement. A small edit just outside the file list (a `mod`
  line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **No `Look-request` trailer** on any commit.
- **Failing tests first,** at the lowest tier that shows the rule: see each fail for the rule's reason, then write the code.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`** conflict across plans: on a catch-up, regenerate rather than
  hand-merge, and keep `names.txt` sorted as it is. Generated files your change makes stale are yours to regenerate.
- **Never raised:** a budget, a timeout or a limit: `tools/tiers/budgets.toml`, `.config/nextest.toml`'s `browser-temper`, a
  step's `within`. Over budget is red.
- **Run one test of the tier:** `heavy python3 -I -B tools/tiers/run temper browser -- -E 'test(=NAME)'`, whose prerequisites
  build the bundles. Once they're built, `heavy cargo nextest run --profile browser-temper -p temper-client-browser-world --test
  browser_all -E 'test(=NAME)'`, with `TEMPER_TEST_BUNDLE` and `TEMPER_TEST_PLAYGROUND` set as `tools/tiers/design.md`, section 1
  has them.
- **Measure the tier:** `heavy python3 -I -B tools/tiers/run --cpu DIR temper browser`, with `DIR` fresh, private and untracked,
  then `tools/tiers/cpu-summary DIR --budgets tools/tiers/budgets.toml --pair browser-temper` (`tools/tiers/design.md`, 7).
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's
  `Emulation.setDeviceMetricsOverride`. A throwaway render script is allowed, untracked and gone before the push. Handle
  `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
