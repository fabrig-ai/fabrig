# look-round-7: the rules every task names

This round lands one look request, `sign-in-new-tab`, by
`temper/docs/design/draft/look-requests.md`, sections 3 to 5. Read this note, then your spec. Where
this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design (`temper/docs/design/client/README.md`, 3.5). A task may
  edit the `docs/design/` files its `Owns and may change` lists. A small edit just outside a task's
  files (a re-export, a `mod` line, a design line, a golden) is made and named in the report.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a
  protocol or a public interface may change freely, with no compatibility and no `Question:`; say so
  in the report. A `Question:` is only for what truly blocks the task.
- **A throwaway render script is allowed** in either task, untracked and gone before the push; the
  render-check command is `temper/docs/design/draft/look-requests.md`, 8, Later: don't build it.

## The request

- `git fetch origin refs/heads/look-requests` gives you the commit: `c725deec96`; should the branch
  have moved, it is the one whose trailer is `Look-request: sign-in-new-tab`. Read
  `git show --format=%B -s c725deec96`, whole: its message is the request. Its own diff is
  `git diff c725deec96^ c725deec96 -- temper/docs/design/client/journeys/`, the source of every rule,
  selector, word and node order. Read a page with
  `git show c725deec96:temper/docs/design/client/journeys/sign-in-new-tab/02.html`.
- **Not to land:** `configure-temper-own-way` above it, held for the user, and `chat-work` below it,
  `look-round-6`'s.
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its
  pages or its `overlay.css`. `sign-in-new-tab/` is hand-made and stays on the person's branch. The
  journeys' tests fail while `overlay.css` holds a byte, so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it.

## The behaviour is not built here

`temper/docs/design/draft/sign-in-new-tab.md` is fabrig/temper's, after this, and takes this look by
catch-up. So nothing here opens a tab, closes one, or changes what a press sends: `Act::SignIn` still
sends its `Return` and the page's own tab still leaves. `root.departing`, `root.returning`,
`Request::Leave`, `temper-client-domain/src/departure.rs` and the listener's words stand as they are.
`Act::SignInElsewhere` and `ConnectModel::elsewhere` stay in the domain, with no entry in the view.

## Commits

**Every commit** ends with a last paragraph of trailers: `Look-request: sign-in-new-tab`, and
directly under it `Co-Authored-By: ...` where your commit carries such a line. No other line comes
between them, and nothing follows the paragraph. A fix, a reprint and a repair of a red carry it
too. Check with `git log --format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.

## `temper.css`

- New rules go under the marker `/* ==== ui: welcome ==== */`, never at the end of the file, so that
  parallel plans merge cleanly. Replaced rules go from where they stand.
- Rules move whole: selectors and declarations as the overlay has them.
- A rule read after an `@media` overrides it at the same specificity: a base rule moved under the
  marker takes its phone rule with it, in an `@media (max-width: 760px)` under the marker.

## The view

- **A class** goes in `temper-client-view/src/look.rs`, its constant and its line in `look::NAMES`,
  as its neighbours; `tests/look.rs` reads that list.
- **A new word** goes in `words.rs` as its neighbours: the `Word` enum, the list `ALL`, its number
  (one unused, above the highest in use), its English; a word no page says any more goes. Bounds
  derived from the list (`SEGMENTS`, `limits.rs`) move by what you add, with their comments, and the
  report says so. They are not budgets.
- **Generated and conflicting files** (goldens, prints, the gallery's `names.txt` and `tests.rs`): on
  a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.

## Rendering

- Headless Chromium (`SKEIN_TEST_CHROMIUM`): Chromium 147's new headless won't go below 500 px by
  `--window-size`, so a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Handle
  `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- The gallery's states are `temper/docs/design/client/browser.md`, section 7.

## Your checks

Besides the implementer's text: a task that touched design text runs
`python3 -I -B tools/citations/citations.py --base origin/main`.

## Your report

Besides what the implementer's text asks: for each state or rule of the request your task landed,
one line, so that the account can be written from the reports.
