# look-round-2: the rules every task names

Every landing of this round reads this note and
`temper/docs/design/draft/look-requests.md`, sections 3 to 6, and follows every step of its
section 3 in order. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text is yours here.** A landing brings the design in line (section 3, steps 2 and 8),
  so a task may edit the `docs/design/` files its `Owns and may change` lists, and the status
  files it names. A draft's own text stays as it is, and so does `docs/help/`.
- **A throwaway render script is allowed,** untracked and gone before the push: the render check
  is each landing's own (section 8 of the draft holds its command, Later: don't build it). Run it
  outside any worktree if you can.
- **Chromium runs directly,** one at a time; it builds nothing, so it needs no `heavy`. Every
  cargo command stays under `heavy`.

## The request

- `git fetch origin refs/heads/look-requests` gives you the commits; no local branch is needed.
  Read `git show <commit>` whole, message and diff, before anything else (section 3, step 1).
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its
  pages as they are or its `overlay.css`. `welcome-ready/` stays on the person's branch. The
  journeys' tests fail while `overlay.css` holds a byte, so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it.

## Commits

- **Every commit** ends with a last paragraph of trailers: `Look-request: NAME`, the task's
  request, and directly under it `Co-Authored-By: ...` where your commit carries such a line. No
  other line comes between them, and nothing follows the paragraph. A fix of what the landing
  missed, a reprint and a repair of a red carry it too: one that lacks it hides the fix from
  `look-refresh.sh` and from whoever reads the request's history. Check with
  `git log --format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.
- **The landing's last commit holds the account** (section 5) in its message, above the trailers:
  one row per overlay rule, per page change and per state the message names, with the render
  check's result under it. Every row is landed, differs and why, or deferred to whom and where it
  is tracked. No row is left open.

## Reprint

- `heavy cargo run -p temper-client-journeys` reprints every journey in place.
- The reprint is compared with the request's pages **as text**:
  `git show <commit>:<page> | diff - <page>`. Each line that differs goes in the account with its
  reason. The later requests take the page as it stands.
- A recording goes stale only if the exchange with the engine changes (`journeys.md`, 8). No
  request of this round does; if `temper-client-journeys` says one has, that's a question.

## The render check (section 4)

Keep everything of it under `~/src/fabrig/look-round-2/<NAME>/`, on your builder, outside any
worktree, `NAME` being the request's. The screenshots stay there and come back to the user; they
are never committed. A later run's `<NAME>` is yours to overwrite.

1. **Two trees of the same shape:** `request/`, from
   `git archive <commit> temper/docs/design/client/journeys temper/crates/temper-client-shell-web`,
   overlay as it stood there; and `landed/`, the same two paths copied from your worktree, overlay
   empty. A page's stylesheets are relative links, so each tree renders on its own.
2. **A measuring copy of each page,** never the page itself: a script before `</body>` that opens
   every `details`, drops `hidden` from each menu and dialog, walks the elements in document order
   and writes, for each and for its `::before` and `::after`, `getComputedStyle` and
   `getBoundingClientRect` as JSON into the document, with `innerWidth`.
3. **Chromium** is `$SKEIN_TEST_CHROMIUM`, else `chromium` on `PATH`:
   `--headless=new --no-sandbox --disable-gpu --dump-dom --virtual-time-budget=3000
   --window-size=W,2400 file:///...` for the figures, and `--screenshot=PATH` with the same width
   for the pictures. Check the `innerWidth` you get. If Chromium floors it above 420, render the
   page in an iframe of that width from a wrapper page, in which the media queries take the
   iframe's width. A value that an animation moves (a `transform`, a `left`) is no difference.
4. **Compare** each element at 1440, 1000 and 420: its computed style and its box, then the
   screenshots, for what styles miss. Every element differing goes in the account with its reason,
   or it's fixed.
5. **States in words** (done, failed, the narrow width, no target) are rendered too, through the
   gallery state the task names or, where it names none, a hand-made page in the scratch directory,
   built from the page beside it and kept with the screenshots. They have no request's page to
   compare with: check them against the message's words, and say so in the account.
6. **Keep:** the request's page beside the landed one for each changed page, at 1440, 1000 and 420,
   with its open states, and each state in words. Names say which is which:
   `07-1440-request.png`, `07-1440-landed.png`, `done-420-landed.png`.

## The terminal (section 3, step 5)

- `skein-ui-text` prints a named region as `NAME:` with its children indented, and a paragraph's
  text runs **joined, with no separator**. A shape the browser stacks by style reads as one run of
  words there, and a mark drawn by style isn't there at all.
- So a part a request stacks or marks by style says it in its terminal's words: the view reads
  `w.shell` (`crate::words::Shell`, as `forge/forges.rs` does).
- Each task's print test goes through `skein_ui_text::render`, as
  `temper-client-view/src/tests/connect_forge.rs`, 1752, does, on `page_view` (the terminal's
  words). `browser_page_view` is the browser's.

## Scope

- **Other plans' work:** `serve-without-box`'s tasks change `pages/welcome.rs`,
  `tests/welcome.rs`, `words.rs` and the terminal's `tests/render.rs` beside you. Change no more of
  those files than your task says, keep your new tests in the new test file it names, and leave
  anything of theirs you meet as it is. `temper/docs/design/draft/claude-provider.md` and its status
  are another change's: leave them.
- **Engine's facts stay deferred:** no change to a stored format, a protocol, or a crate outside
  `temper-client-view`, `temper-client-shell-web`, the gallery and their tests. An engine's fact
  that a request needs is section 6's: it's named in the account, deferred, in the status your task
  names; it's never built.
- **A conflict with a design or a test the landing cannot settle** (a test that walks a part the
  request removes, a design choice it reverses) is a question to the user, never your choice
  (section 3, step 2).

## Your checks

- After each change, the narrowest test; at the end, once, for each landing:
  `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-journeys -p temper-client-shell-terminal -p temper-client-world`,
  and `heavy cargo clippy` over the same packages, then
  `python3 -I -B tools/citations/citations.py --base origin/main`, since each landing changes
  design text.
- A citation of a document not on `main` is a broken citation: name such a draft in words, without
  its path.

## Your report

Besides what the implementer's text asks: the account table whole, the render check's result, and
which builder (`hostname`) keeps the screenshots, with the directory.
