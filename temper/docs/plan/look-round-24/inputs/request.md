# look-round-24: the ways page's render-check misses

A look plan, in place of `look-round-23`, whose cut failed when Claude's limit hit, so nothing of it was built: the render check after `look-ask-ways` landed, for `Look-request: ask-temper-ways` on `origin/look-requests`, found by its trailer. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5. Everything else passes: the opening, the order, "now", the words, the live acts ("Use this", the refusal, "Keep it", "Yours"), the paths derived from the settings, and the phone's vertical path.

## Tasks

1. **No outline on the path's drawing.** Every stop's hit area, star, code square, main's bar and SVG label gains a 1 px ink stroke (`rgb(24,24,27)` by day, `rgb(250,250,250)` by night), which makes the labels look bold. It comes from skein's `[data-drawn] > svg :is(g,line,polyline,path,rect,circle,text)` (`skein/crates/skein-ui-bundle/skein-ui.css`, near line 25). The request has `stroke: none` on them. Fix it in temper's own rules for the path, scoped to it, leaving skein's rule as it is.
2. **The gallery at 420 runs the path down the page,** as live does. Today it squeezes the 720×140 scene to 75 px high, with 8 px labels.
3. **At 1000, the path scales as the request's does:** 658×127.94, not redrawn at 658×140. Today that puts the open card 12 px low.
4. **The open card's spacing on a phone:** the request's 14 px gap above the card. Live it sits 14 px higher. The phone's caret stays hidden, as in the request.
5. **A card shows its charter's first sentence whole,** as the request does: "Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against what the change was asked to do." Today the 128-byte fact bound cuts it mid-word, at "…judge them against wh". Size that fact's bound to hold a first sentence. Past the bound, cut on a word with "…".
6. **"Open the reviewer ›" opens the reviewer's charter,** not Settings' general How work runs. The same goes for every card's charter or rule link (`temper/crates/temper-client-view/src/pages/ask_page.rs`, near line 214).
7. **The CI card names the repository's CI workflows,** as the request's unpictured states ask, not "the repository's CI" alone.
8. **The terminal prints "Ways of working:" and "The way a change goes:" once each.** Today `temper open` repeats each.

A browser case is a row of an existing table test, within the cap of 12. The gallery's prints are regenerated as the common notes say.

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/stock-ways.md`


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
