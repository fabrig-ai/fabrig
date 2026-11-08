# look-round-20: To begin's Home, Ask's tabs, the switch's place and the night's shadows

A look plan: the render check after `look-round-19` landed. The requests are `to-begin-two-steps` (`4332a26265`) and `night-mode` (`3c5333485a`) on `origin/look-requests`, found by their trailers. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5. The night images the CSP blocked are `red-74`'s repair, already on main, so they aren't this plan's.

## Tasks

1. **Home's Needs you and Chats stay out while To begin shows,** at every width, as `to-begin-two-steps` says. The gallery's `home-needs-under-begin` and `home-begin-settings-left` show "NEEDS YOU · 1" and "CHATS" under the card. Live, a chat started elsewhere shows under it too, with either step left. A row of an existing table test holds it.
2. **Ask's restricted tabs on a phone.** At 420, Ask temper's page as Start opens it shows no bottom tab bar. It should have the three equal tabs, Home, Ask and Settings, as Home has while To begin is unfinished.
3. **The night switch is always there,** as `night-mode` places it: at the rail's foot, and at the top bar's right on a phone. Today Ask's switch is a 0×0 box at 420, because its rail is hidden. A chat with its draft in Split hides the switch at every width, when the rail collapses. Give each a visible switch in the request's place for that layout.
4. **The night's shadows are the request's.** The landed night sets every shadow to `rgba(0,0,0,.4)`. The request keeps each one's own colour:
   - the project picker and person menu: `0 12px 32px rgba(24,24,27,.12)`;
   - the model menu: `… .14`;
   - starter and scale cards: `0 1px 2px rgba(24,24,27,.04)`;
   - the selected Split: `0 1px 2px rgba(0,0,0,.08)`;
   - Latest: `0 2px 8px rgba(24,24,27,.1)`.

   Match them, through tokens.

The browser tier's cap of 12 holds. The gallery's states are regenerated as the common notes say.

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
