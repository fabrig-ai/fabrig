# look-round-11: the rules every task names

This round lands one look request, `model-picker`, by `temper/docs/design/draft/look-requests.md`, sections 3 to 5. Read
this note, then your spec. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design (`temper/docs/design/client/README.md`, 3.5, *The work's account and model*,
  5.5). A task may edit the `docs/design/` files its `Owns and may change` lists. A small edit just outside a task's files
  (a re-export, a `mod` line, a design line, a golden) is made and named in the report.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public
  interface may change freely, with no compatibility and no `Question:`; say so in the report. A `Question:` is only for
  what truly blocks the task.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories,
  or a tier's CPU figure. A new story for those tiers is written and compiled, not run. A look's render check by a throwaway
  driver isn't a tier.
- **A throwaway render script is allowed,** untracked and gone before the push; the render-check command is
  `temper/docs/design/draft/look-requests.md`, 8, Later: don't build it.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping
  what it checks for everything else, and named in the report. **Generated files** your change makes stale (prints,
  goldens, `docs/help/reference/*`, counted baselines) are yours too, but for the journeys' prints, which are
  `picker-proof`'s.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and
  your branch fails, it's yours.
- **Help pages:** a `docs/help/` page your change grows past its bound is yours to shorten, losing no fact, never raising
  the bound.

## The request

- `git fetch origin refs/heads/look-requests` gives you the commit: `47c23ff6ed`; should the branch have moved, it is the
  one whose trailer is `Look-request: model-picker`. Read `git show --format=%B -s 47c23ff6ed`, whole: its message is the
  request. Its own diff is `git diff 47c23ff6ed^ 47c23ff6ed -- temper/docs/design/client/journeys/`, the source of every
  rule, selector, word and node order. Read a page with `git show
  47c23ff6ed:temper/docs/design/client/journeys/model-picker/02.html`.
- **Pages:** 01 closed on a new chat; 02 open, Claude alone; 03 open, both; 04 All models; 05 Configure temper; 06 a started
  chat, open; 07 Settings, Models, **not landed here**.
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its pages or its
  `overlay.css`. `model-picker/` stays on the person's branch. The journeys' tests fail while `overlay.css` holds a byte,
  so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it. Other requests on it are other
  plans'.

## What this round doesn't land

Settings, Models (07): its tab, list, switches, "Read again", "Add a model by name", and its two states in words, "Couldn't
read OpenAI's list" and "estimated prices". They are steps 2 and 3 of `temper/docs/design/draft/model-catalogue.md`. So
there is no tab between Accounts and Builders; All models draws no "new" and its link goes to Accounts, saying "Accounts,
in Settings". The overlay's `/* Settings' list of models. */` rules stay out. Nothing here changes which models come with
temper, the default order, or `me`: `model-catalogue-1` landed the facts.

## Beside other plans

`ui-proposal-card` is changing `temper.css` and `pages/chat.rs`, and `model-check` re-records `chat-draft`. In
`temper/crates/temper-client-view/src/words.rs`, `look.rs`, `limits.rs`, `pages/chat.rs` and `temper.css`, keep each hunk as
small as the change allows: add and remove lines, reorder and reformat nothing. A print conflict at catch-up is regenerated,
not merged; so are goldens, `names.txt` (kept sorted as it is) and the gallery's `tests.rs`.

## Commits

**Every commit** ends with a last paragraph of trailers: `Look-request: model-picker`, and directly under it
`Co-Authored-By: ...` where your commit carries such a line. No other line comes between them, and nothing follows the
paragraph. A fix, a reprint and a repair of a red carry it too. Check with `git log
--format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.

## `temper.css`

- New rules go under the marker `/* ==== ui: chats ==== */`, never at the end of the file, so that parallel plans merge
  cleanly. Replaced rules go from where they stand.
- Rules move whole: selectors and declarations as the overlay has them; what must differ is named in your report for the
  account.
- A rule read after an `@media` overrides it at the same specificity: a base rule moved under the marker takes its phone
  rule with it, in an `@media` under the marker.

## The view

- **A class** goes in `temper-client-view/src/look.rs`, its constant and its line in `look::NAMES`, as its neighbours;
  `tests/look.rs` reads that list. **A retired class** keeps its slot as `temper-reserved-N`, as `look::NAMES` shows, and
  its constant goes.
- **A new word** goes in `words.rs` as its neighbours: the `Word` enum, the list `ALL`, its number, its English; a word no
  page says any more goes. Parallel plans are adding words and look tokens: start your numbers 40 above the highest in use,
  and on a catch-up that collides, take others. Bounds derived from the list (`SEGMENTS`, `limits.rs`) move by what you add,
  with their comments, and the report says so. They are not budgets.

## Rendering

- Headless Chromium (`SKEIN_TEST_CHROMIUM`): Chromium 147's new headless won't go below 500 px by `--window-size`, so a 420
  px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's
  `onbeforeunload` hangs a CDP driver.
- The gallery's states are `temper/docs/design/client/browser.md`, section 7.

## Your checks

Besides the implementer's text: a task that touched design text runs `python3 -I -B tools/citations/citations.py --base
origin/main`.

## Your report

Besides what the implementer's text asks: for each state or rule of the request your task landed, one line, so that the
account can be written from the reports.
