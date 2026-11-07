# look-round-13: Settings' connect pages become the welcome's, and "Add repositories"

The user's go, about 22:00Z on 2026-11-06, relayed from their laptop: land the look request `connect-flows` (`97ad67393b`, on `origin/look-requests`; the branch moves, and the user may stack a change on the same commit, so read the branch now and find it by its `Look-request: connect-flows` trailer). Pages and notes in `temper/docs/design/client/journeys/connect-flows/`: its README says what each page is, what goes and what isn't mocked. Make the real pages look exactly like these.

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message. Other requests on the branch are other plans'.

## What it is

1. **Settings' connect pages, 01 to 15:** Settings takes the welcome's cards, panels and words under its own path and column. The user approved these.
2. **"Add repositories", 16 to 23:** the sheet of `temper/docs/design/draft/repository-copies.md`, 9, over Settings' Repositories, in place of the Adopt page. Its three sources are Adopt, with the search list the user approved as 16.html, Make empty and Copy public. It's folded in here so adopting isn't built twice. Failed copies get their "Copy again" and "Dismiss" here.

## Decisions that come with it

Taken on the laptop's recommendations; the user can reverse any:
- The demo API key goes.
- "Use deployment accounts" becomes one quiet link.
- "All ways" goes from the welcome: it goes to the same place as "‹ Back".
- Search and scroll replace "Next" in `temper/docs/design/client/forge.md`, 5.1: change 5.1 with it.
- The forge sits inside "Adopt", not above the sources, since a repository is only made or copied on a Forgejo: change `repository-copies.md`, 9, with it.
- Settings' "Forge number" goes (`temper-client-domain/src/forge/words.rs`).

## How the cut shapes it

Started after `model-check` landed, as it changed the same connect pages; `model-catalogue-2`'s Models tab and `look-round-11` are on Settings beside it, and whichever lands second catches up. Screenshots beside the request's pages at 1440, 1280, 1000 and 420 wide, in `~/src/fabrig/look-round-13/` on the builder, outside any worktree, the report saying which builder. Journeys whose prints this changes are regenerated with `temper-client-journeys again`, as below. A browser story changed by this is compiled, not run.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/repository-copies.md`
- `temper/docs/design/client/forge.md`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
