# look-round-11: the model and effort sit at the composer, on every chat

At the user's request (2026-11-06, on journeys 8 and 9: "the model picker also needs some love re looks", "the configuration chat should have a model picker too", "in our picker we should offer the most popular", "We're also missing the ability to pick the effort level."): land the look request `model-picker` (`47c23ff6ed`, on `origin/look-requests`; the branch moves, so if that hash is gone, find it by its `Look-request: model-picker` trailer). Pages `journeys/model-picker/01.html` to `07.html`; every state is in its message.

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message. Every commit carries `Look-request: model-picker` directly above `Co-Authored-By:`. Other requests on the branch are other plans'.

**It's `temper/docs/design/draft/model-catalogue.md`'s step 1, whose facts are on main** from `model-catalogue-1`: the offer and `resolved_default` in `me`, efforts, the configuring chat's picker. Build on what that plan landed; where it drew a stopgap of the picker, this replaces it.

## What this round lands, and what it doesn't

- **Lands:** closed and open (01 to 03), All models (04), Configure temper (05), a started chat (06), and the message's "States the pages do not show", the terminal's included. With one account, the message's last paragraph and `model-catalogue.md`, section 3, hold everywhere: no model of a provider with no usable account shows.
- **Removes:** the message's "What it replaces" list, rules and view both.
- **Doesn't land: Settings, Models (07).** Its list, switches, "Read again" and "Add a model by name" are steps 2 and 3 of `model-catalogue.md`, which come after. So there's no tab between Accounts and Builders yet. On All models, "new" isn't drawn, as nothing reads a provider's list yet, and its link to Settings goes to Accounts. In the plan's account, say that 07 and these two stay for step 2.
- **Not the look's:** which models come with temper, the default order, the list in the store, reading a provider's list. `model-catalogue.md` settles them.

## How the cut shapes it

Three tasks at most, the view and the stylesheet first. Screenshots beside the request's pages at 1440, 1280, 1000 and 420 wide, menu open and closed, in `~/src/fabrig/look-round-11/` on the builder, outside any worktree, the report saying which builder. Journeys whose prints this changes are regenerated with `temper-client-journeys again`, as below. A browser story for the picker is written and compiled, not run: one menu open at 420, a model pressed on a started chat, an effort pressed.

`model-check` is on its way to main with `chat-draft`'s prints re-recorded. A print conflict at catch-up is regenerated, not merged.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/model-catalogue.md`
- `temper/docs/design/draft/model-catalogue.status.md`
- `temper/crates/temper-client-view/src/pages/session.rs`

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
