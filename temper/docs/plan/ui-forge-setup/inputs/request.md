# ui-forge-setup: skein's forge setup says who a forge account is

The second piece of `temper/docs/design/draft/forge-identity.md`, on the user's word: section 4's generic part, in skein, as a plan of its own before temper's story. Section 3's hole is closed (`ui-forge-reviewer`). temper's link, its record and its card are the next plan: don't build them here, and no system or product is named in skein.

## What this plan makes true

1. **`Reconcile` is given a login and answers an id** (4.1; skein's `docs/design/forge-setup.md`, 4.1 and 4.3): `Reconcile { forge, adopted }` is also given the person's login, reads `GET /users/<login>` on the set-up's admin token, and answers its `id` in `Done`. With no login given, it reads nothing, as a fake forge's. An account gone isn't a failure: it answers none.
2. **`owner` is decoded from `GET /app`** (4.2; `forge-setup.md`, 5.2 and 5.6; `forge-github.md`), with `id`, `login` and `type`, in `Convert` and `Reconcile`, each of which already reads it and checks the App's id. It's answered to the caller. A read that fails changes nothing and reports it, as any. The live conformance adds it: written and compiled, not run, as it hasn't been measured on `GET /app` itself.
3. **A new secret item, `AppMade`** (4.2; `forge-setup.md`, 6.4): `Convert` writes it first, its bytes the App's id, as `Copying` is written first, and it's kept in the secrets' store with `AppId`.

Read section 4.1, 4.2 and 8 before cutting, for why these and not `GET /apps/<slug>` or a person task. `forge-setup.md` and `forge-github.md` change with the code, and `draft/forge-identity.status.md`'s 4.1 and 4.2 lines go. temper's callers keep compiling and ignore the new answers, or pass no login, until the next plan.

## Limits

One or two tasks. An interface change in skein's forge setup: none of the crates that need a word to hearth/core, by my reading, but say if one is. Focused and fuzzy alone, as below: no browser, walk or render check, and no live run. New cases are rows of an existing table test, and skein's budgets apply.

## Sources

- `temper/docs/design/draft/forge-identity.md`
- `skein/docs/design/forge-setup.md`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
