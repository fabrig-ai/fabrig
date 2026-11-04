# look-round-4: the rules every task names

Every task of this round lands part of a look request on `origin/look-requests`, by
`temper/docs/design/draft/look-requests.md`, or builds what such a request needs from
`temper/docs/design/draft/to-begin.md`. Read this note, then
`temper/docs/plan/look-round-4/notes/contracts.md`, then your spec. Where this note and the
implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit only the `docs/design/` files its
  `Owns and may change` lists: its status entries as it closes them, and the journeys' pages it
  reprints. A draft's own text and `docs/help/` stay as they are.
- **A throwaway render script is allowed,** untracked and gone before the push, in the `account-*`
  tasks alone; the render-check command is `temper/docs/design/draft/look-requests.md`, 8, Later:
  don't build it.
- **Screenshots** go to `~/src/fabrig/look-round-4/<request name>/` on your builder, outside any
  worktree, and are never committed; your report names the builder.

## The requests

| Request | Commit | Over |
|---|---|---|
| `project-home-everyday` | `8bbc1bc964` | `project-home`, landed |
| `project-home-onboarding` | `9fcbec1d32` | `project-home-everyday` |
| `to-begin-more-accounts` | `0e816f6749` | `project-home-onboarding` |
| `home-drops-connected` | `2c6f5a5146` | `to-begin-more-accounts` |
| `configure-chat-in-settings` | `53ba7876f8` | `home-drops-connected` |

- `git fetch origin refs/heads/look-requests` gives you the commits; no local branch is needed.
  `git show --format=%B -s COMMIT` is a request's whole message. A request's own diff is
  `git diff COMMIT^ COMMIT -- temper/docs/design/client/journeys/`, the source of every rule,
  selector, word and node order a task lands. Read it with `git show COMMIT:PATH`; the pages as the
  request draws them are at its commit, and the hand-made pages (`project-at-work/`,
  `project-after-skip/`, `project-switcher/`, `configure-chat/`) are read there too.
- **Held, never landed:** `new-menu` (`851708d039`) and `configure-chat-draft` (`d40071cd81`).
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its
  pages as they are or its `overlay.css`. The hand-made pages stay on the person's branch. The
  journeys' tests fail while `overlay.css` holds a byte, so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it.

## Commits

- **Every commit** ends with a last paragraph of trailers: the `Look-request: NAME` your spec names,
  and directly under it `Co-Authored-By: ...` where your commit carries such a line. No other line
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
  replaced are removed. A rule's commit is the request's, so read the overlay at its commit.
- **The terminal:** `skein-ui-text` prints a named region as `NAME:` with its children indented,
  and a paragraph's text runs **joined, with no separator**. A shape the browser stacks by style
  reads as one run of words there, and a mark drawn by style isn't there at all. So a part stacked
  or marked by style says it in the terminal's words: the view reads `w.shell`
  (`crate::words::Shell`), as `pages/home.rs`'s `step` does. The print tests go through
  `skein_ui_text::render` on `page_view` (the terminal's words); `browser_page_view` is the
  browser's. A heading the browser drops by style stays a heading.
- **A new word** goes in `words.rs` as its neighbours: the `Word` enum, the list, its number (one
  unused), its English. The list's bounds in `limits.rs` are derived: raise a derived count by what
  you add, with its comment, and say so. It is not a budget. Tasks side by side take their numbers
  from `notes/contracts.md`.

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
one line, so that the request's `account-*` task can be written from the reports.
