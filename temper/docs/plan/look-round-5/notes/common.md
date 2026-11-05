# look-round-5: the rules every task names

This round lands one look request, `configure-temper-name`, by
`temper/docs/design/draft/look-requests.md`, sections 3 to 5. Read this note, then your spec. Where
this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design (`temper/docs/design/client/README.md`, 3.5 and 5.13). A
  task may edit the `docs/design/` files its `Owns and may change` lists. A small edit just outside
  a task's files (a re-export, a `mod` line, a design line, a golden) is made and named in the report.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a
  protocol or a public interface may change freely, with no compatibility and no `Question:`; say so
  in the report. A `Question:` is only for what truly blocks the task.
- **A throwaway render script is allowed** in either task, untracked and gone before the push; the
  render-check command is `temper/docs/design/draft/look-requests.md`, 8, Later: don't build it.

## The request

- `git fetch origin refs/heads/look-requests` gives you the commit: `cea9fdb5d2`; should the branch
  have moved, it is the one whose trailer is `Look-request: configure-temper-name`. Read
  `git show --format=%B -s cea9fdb5d2`, whole: its message is the request. Its own diff is
  `git diff cea9fdb5d2^ cea9fdb5d2 -- temper/docs/design/client/journeys/`, the source of every rule,
  selector, word and node order. Read the page with
  `git show cea9fdb5d2:temper/docs/design/client/journeys/configure-chat/01.html`.
- **Held, never landed:** `new-menu`, `configure-chat-draft` and `configure-temper-own-way`.
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its
  pages or its `overlay.css`. `configure-chat/` is a hand-made page and stays on the person's branch.
  The journeys' tests fail while `overlay.css` holds a byte, so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it.

## Commits

**Every commit** ends with a last paragraph of trailers: `Look-request: configure-temper-name`, and
directly under it `Co-Authored-By: ...` where your commit carries such a line. No other line comes
between them, and nothing follows the paragraph. A fix, a reprint and a repair of a red carry it
too. Check with `git log --format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.

## `temper.css`

- New rules go under the marker `/* ==== ui: settings ==== */`, never at the end of the file, so
  that parallel plans merge cleanly. Replaced rules go from where they stand.
- Rules move whole: selectors and declarations as the overlay has them.
- A rule read after an `@media` overrides it at the same specificity: a base rule moved under the
  marker takes its phone rule with it, in an `@media (max-width: 760px)` under the marker.
- `ui-chats`, landing soon beside this, takes the `.temper-session` layout to every chat. Keep the
  names plain: no rule here names the configuring page where it is the layout's.

## The view

- **A class** goes in `temper-client-view/src/look.rs`, its constant and its line in `look::NAMES`,
  as its neighbours; tests read that list.
- **A new word** goes in `words.rs` as its neighbours: the `Word` enum, the list `ALL`, its number
  (one unused, above the highest in use), its English; a word no page says any more goes. Bounds
  derived from the list (`SEGMENTS`, `limits.rs`) move by what you add, with their comments, and the
  report says so. They are not budgets.
- **The terminal:** `skein-ui-text` prints a named region as `NAME:` with its children indented, and
  a control's text runs **joined, with no separator**. A shape the browser stacks or marks by style
  says it in the terminal's words: the view reads `w.shell` (`crate::words::Shell`), as
  `pages/home.rs`'s `step` does. Print tests go through `skein_ui_text::render` on `page_view` (the
  terminal's words); `browser_page_view` is the browser's.
- **skein-ui's web shell** writes a button's name as its text, so a button with spans of its own
  has no name: its children name it, and that is what the request draws.

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
