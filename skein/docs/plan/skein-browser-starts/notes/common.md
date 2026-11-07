# skein-browser-starts: the rules every task names

The request is `skein/docs/plan/skein-browser-starts/inputs/request.md`: its *Limits* and *For every task of this plan* bind every task, and win over anything here that seems to differ. `skein/docs/plan/skein-browser-starts/plan.md` has the table, one row per test: a row is your contract for where each of its assertions goes.

## What runs

- **The browser tier is not run, not a story of it nor by name.** No `tools/tiers/run`, no `--profile browser-skein`, no `skein/scripts/browser.sh`, no `SKEIN_TEST_CHROMIUM`. A merged browser test is written and compiled: `heavy cargo check -p PKG --tests`, and `heavy cargo nextest run -p PKG --no-run`. Say so in the report. After-landing runs it, and a red comes back as a repair.
- **A lower story runs,** in focused, by name, through `heavy` (`AGENTS.md`, 5): it exists and passes there before the browser test that held its assertion goes, on the same branch.
- Before you call a focused or fuzzy failure "outside the change", run it once on the pristine base. If main passes and your branch fails, it's yours.

## What a merge keeps

- **Every assertion stays,** as a step, a case or a lower story, as the row says. An assertion may change only in the words a merge forces: a step's number, an address that counts the notes before it, a title made distinct so that a find can't pass on an earlier story's note. Each such change is named in the report.
- **No deadline widens.** Each step keeps its `within` and each `Duration` stays. No test sets `Harness::horizon`. Every run stands on the harness's forty seconds and every `Rig` on its one minute. If a merged test would plainly need longer, it stays two tests, and say so. Nothing here measures that: a merge's size is the plan's reading, so make no longer merge than your spec's.
- **A merged test says which story's step failed:** `Outcome::Failed`'s `person` where each story is a person, `Harness::run_stories`' report where they take turns, the case's name in a `Rig` (`notes/contracts.md`).
- **The kit holds four people in a run** (`skein_browser::Limits::persons`). A fifth is refused: stories past four take turns.
- **The example's server keeps its notes for the whole run,** so stories that share a run share them. Read the row before you reorder.
- **A helper a merge leaves unused goes in the same task.**

## What stays out

- **No product code:** `skein-ui-*` and `skein-browser`'s behaviour don't change. A product bug goes in the report.
- **`skein_fake_person_browser::stories` and `Setup` keep their shapes:** jig's and temper's tiers call them. `Harness` gains only what `notes/contracts.md` adds.
- **Budgets and timeouts are the user's:** `tools/tiers/budgets.toml` and `.config/nextest.toml` don't change.
- **Other plans add stories to `skein/tests/ui-web`.** If the trunk holds a test the table doesn't, keep it working and name it in the report.

## Design and status

The cut wrote `skein/docs/design/ui-web.md`, 14, and `skein/docs/design/browser.md`, 6 and 9. **A departure from your run's text:** you remove the **Not yet** line your spec names from its status file, or edit it as your spec says, and touch nothing else in `docs/design/`. What the code taught goes in the report.
