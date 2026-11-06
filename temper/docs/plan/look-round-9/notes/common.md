# look-round-9: the rules every task names

This round lands one look request, `model-check`, by `temper/docs/design/draft/look-requests.md`,
sections 3 to 5. Read this note, then your spec. Where this note and the implementer's text differ, this
note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design (`temper/docs/design/client/README.md`, 3.5, *The try*). A
  task may edit the `docs/design/` files its `Owns and may change` lists. A small edit just outside a
  task's files (a re-export, a `mod` line, a design line, a golden) is made and named in the report.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol
  or a public interface may change freely, with no compatibility and no `Question:`; say so in the
  report. A `Question:` is only for what truly blocks the task.
- **A throwaway render script is allowed** in either task, untracked and gone before the push; the
  render-check command is `temper/docs/design/draft/look-requests.md`, 8, Later: don't build it.

## The request

- `git fetch origin refs/heads/look-requests` gives you the commit: `0bfb77711b`; should the branch have
  moved, it is the one whose trailer is `Look-request: model-check`. Read `git show --format=%B -s
  0bfb77711b`, whole: its message is the request. Its own diff is `git diff 0bfb77711b^ 0bfb77711b --
  temper/docs/design/client/journeys/`, the source of every rule, selector, word and node order. Read a
  page with `git show 0bfb77711b:temper/docs/design/client/journeys/model-check/03.html`.
- **Not to land:** `configure-temper-own-way` above it, `look-round-8`'s, and `sign-in-new-tab` below it,
  `look-round-7`'s. The overlay at this commit holds their rules too: this request's rules are its own
  diff's, under the comment `/* model-check: ... */`.
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its pages
  or its `overlay.css`. `model-check/` is hand-made and stays on the person's branch. The journeys' tests
  fail while `overlay.css` holds a byte, so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it.

## The behaviour is not built here

`temper/docs/design/draft/model-check.md` is fabrig/temper's, after `first-usable-model`, and takes this
look by catch-up. So no trial runs here: no task, no charter, no new request on the wire, no stored fact.
`welcome_program_progress` still moves a usable model on to Ready. Where the view has no fact, the two
functions of `temper-client-view/src/pages/model_check.rs` stand for it, and the account names them.

## Chained behind `look-round-7`

`look-round-7` (`sign-in-new-tab`) changes the same Model step and is landing now. It takes
`welcome_model.rs`'s `note`, `paste` and `numbered`, `ModelStep::Paste`, the card's entry, the words
`WelcomeAway*`, `WelcomePaste*` and `WelcomeModelSignInTime`, and adds `pages/sign_in_wait.rs`. Those are
its: whether or not they are on your trunk yet, leave them, and keep its words. Write in `program`,
`usable` and the one branch of `connect_model::body` your spec names. On a catch-up that brings it in,
keep its side of a conflict in its own functions, and regenerate rather than hand-merge what is generated.

## Commits

**Every commit** ends with a last paragraph of trailers: `Look-request: model-check`, and directly under
it `Co-Authored-By: ...` where your commit carries such a line. No other line comes between them, and
nothing follows the paragraph. A fix, a reprint and a repair of a red carry it too. Check with `git log
--format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.

## `temper.css`

- New rules go under the marker `/* ==== ui: welcome ==== */`, never at the end of the file, so that
  parallel plans merge cleanly. Replaced rules go from where they stand.
- Rules move whole: selectors and declarations as the overlay has them.
- A rule read after an `@media` overrides it at the same specificity: a base rule moved under the marker
  takes its phone rule with it, in an `@media (max-width: 760px)` under the marker.

## The view

- **A class** goes in `temper-client-view/src/look.rs`, its constant and its line in `look::NAMES`, as its
  neighbours; `tests/look.rs` reads that list.
- **A new word** goes in `words.rs` as its neighbours: the `Word` enum, the list `ALL`, its number, its
  English; a word no page says any more goes. Parallel plans are adding words and look tokens: start your
  numbers 40 above the highest in use, and on a catch-up that collides, take others. Bounds derived from
  the list (`SEGMENTS`, `limits.rs`) move by what you add, with their comments, and the report says so.
  They are not budgets.
- **Generated and conflicting files** (goldens, prints, the gallery's `names.txt` and `tests.rs`): on a
  catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.

## Rendering

- Headless Chromium (`SKEIN_TEST_CHROMIUM`): Chromium 147's new headless won't go below 500 px by
  `--window-size`, so a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Handle
  `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- The gallery's states are `temper/docs/design/client/browser.md`, section 7.

## Your checks

Besides the implementer's text: a task that touched design text runs `python3 -I -B
tools/citations/citations.py --base origin/main`.

## Your report

Besides what the implementer's text asks: for each state or rule of the request your task landed, one
line, so that the account can be written from the reports.
