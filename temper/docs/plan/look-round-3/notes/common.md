# look-round-3: the rules every task names

Every task of this round lands part of the look request `project-home`, commit `0197eaea31` on
`origin/look-requests`. Read this note, then `temper/docs/plan/look-round-3/notes/contracts.md`,
then your spec. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit only the `docs/design/` files its
  `Owns and may change` lists: its status entries as it closes them, and the journeys' pages it
  reprints. A draft's own text and `docs/help/` stay as they are.
- **A throwaway render script is allowed,** untracked and gone before the push (the last task
  alone; the render-check command is the draft's section 8, Later: don't build it).

## The request

- `git fetch origin refs/heads/look-requests` gives you its commits; no local branch is needed.
  `git show 0197eaea31` shows the message whole and the overlay and pages, the source of every
  rule, selector, word and node order a task lands.
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its
  pages as they are or its `overlay.css`. `project-switcher/` stays on the person's branch. The
  journeys' tests fail while `overlay.css` holds a byte, so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it.

## Commits

- **Every commit** ends with a last paragraph of trailers: `Look-request: project-home`, and
  directly under it `Co-Authored-By: ...` where your commit carries such a line. No other line
  comes between them, and nothing follows the paragraph. A fix, a reprint and a repair of a red
  carry it too: one that lacks it hides the fix from `look-refresh.sh` and from whoever reads the
  request's history. Check with
  `git log --format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.

## Rules of the page

- **Selectors the view can carry:** the overlay's rules select by the `temper-` names, roles and
  regions. Each class a task draws goes in `temper-client-view/src/look.rs` (`look::NAMES`), as its
  neighbours do; tests read that list.
- **Rules move whole:** where a task lands overlay rules, selectors and declarations go into
  `temper.css` as the overlay has them, with their comments, and the rules the task names as
  replaced are removed.
- **The terminal:** `skein-ui-text` prints a named region as `NAME:` with its children indented,
  and a paragraph's text runs **joined, with no separator**. A shape the browser stacks by style
  reads as one run of words there, and a mark drawn by style isn't there at all. So a part stacked
  or marked by style says it in the terminal's words: the view reads `w.shell`
  (`crate::words::Shell`), as `forge/connect.rs`'s `stacked` does. The print tests go through
  `skein_ui_text::render` on `page_view` (the terminal's words); `browser_page_view` is the
  browser's.
- **A new word** goes in `words.rs` as its neighbours: the `Word` enum, the list, its number (one
  unused), its English. The list's bounds in `limits.rs` are derived: raise a derived count by what
  you add, with its comment, and say so. It is not a budget.

## Scope

- **No protocol, no store:** no change to a stored format, to `client-wire-v1.schema`, to jig's
  contract, or to a crate outside temper's client and its tests, unless the spec names it. An
  engine's fact a request needs is `temper/docs/design/draft/look-requests.md`, 6: it's deferred in
  the status the spec names, never built.
- **Other plans' work beside you:** see your spec's `Owns and may change`. Leave what is theirs.
- **A conflict with a design or a test you cannot settle** (a test that walks a part the request
  removes, in a way your spec doesn't cover) is a `Question:`, never your choice.

## Your checks

Besides the implementer's text: each landing that touched design text runs
`python3 -I -B tools/citations/citations.py --base origin/main`.

A change of a page's markup reprints: `heavy cargo run -p temper-client-journeys` reprints every
journey in place, and the task commits the pages it changed. A recording goes stale only if the
exchange with the engine changes (`temper/docs/design/client/journeys.md`, 8): a task that stales
one says so in its report and leaves it to `record-landing`, unless its spec says otherwise.

## Your report

Besides what the implementer's text asks: for each state or rule of the request your task landed,
one line, so that the last task's account can be written from the reports.
