# look-outside-pages: every page outside a project in the welcome's look

A look plan, the user's of 2026-11-08: the old look is to be gone for good from every page outside a project. The request is `Look-request: outside-pages` on `origin/look-requests` (`43a18a5042`). Its whole message is the request: the frame, the column, each page, the server's pages, the boot, what leaves `look.rs` and `temper.css`, and the tests that keep the old look out. Its pages are `temper/docs/design/client/journeys/outside-pages/01.html` to `10.html`, with `pages.css` and the overlay's rules. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5. Build every state the message names, the ones its pages don't draw among them.

**On the happy path, not only a look:** on a Google deployment, a signed-out `/terminal?code=…` today says to open a sign-in link temper printed, and such a deployment never prints one, so a person signing a terminal in is stuck. Page 05's "Continue with Google", with `then` set to the page's address and its code, fixes that. With no Google sign-in, today's words stay.

Its tests are the message's, focused:
- each place outside a project draws `TOP` and the column;
- the approval's signed-out state with Google offered gives the Google link with its `then`;
- the Google endings link `pages.css`;
- every class `temper.css` selects is one `look.rs` gives;
- no source of the bundle or of the server's pages names `#b8664e`.

A browser case is a row of an existing table test, within the cap of 12.

The web's approval page is shared with the TUI's owner (it was told first): its states and words change here, and its codes, acts and `then` stay as they are.

## Sources

- `temper/docs/design/draft/look-requests.md`


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
