# ui-chat-draft: the rules every task names

The request is `temper/docs/plan/ui-chat-draft/inputs/request.md`: read its opening, *The mock and the requests*,
*Decisions already taken*, the section your spec names, and *For every task of this plan*, which binds you and is not
repeated here. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit only the status entries it closes, in the design status
  files its spec names, and the journeys' prints. If the code teaches that a design line is wrong, make the small edit,
  and name it in your report.
- **Decide, don't ask:** a `Question:` is only for what truly blocks the task. A stored format, a protocol or a public
  interface may change freely, in place, with no version bump and no compatibility. A small edit just outside your files
  (a re-export, a `mod` line, a design line, a golden) is made and named in your report.
- **Render proofs:** a throwaway render script is allowed, untracked and gone before the push. Screenshots go to
  `~/src/fabrig/ui-chat-draft/<the look request's name, or your task>/` on your builder, outside any worktree, never
  committed; your report names the builder.

## The contracts

`temper/docs/plan/ui-chat-draft/notes/contracts.md` holds the shapes that cross tasks. They bind: use its names and
fields. What it leaves open inside a module is yours.

## The look

- **The rule to copy:** a look request's overlay rule (`git show 727f3523ee` for `configure-chat-draft`, `git show
  4194189e15` for `new-menu`: their `temper/docs/design/client/journeys/overlay.css`), or else the selector the design
  names in `temper/docs/design/client/greenfield/mock.css`, pixel for pixel, onto the client's own classes, in the `--gf-*`
  tokens. The mock's markup is `temper/docs/design/client/greenfield/app.js`. Where a look request and the mock differ the
  request wins, as *The mock and the requests* says.
- **`temper.css`:** new rules go under `/* ==== ui: chats ==== */`, the rail's under its own area's marker if it has one,
  never at the end of the file. Remove replaced rules where they stand.
- **The terminal is the same view.** What the mock lacks and a print needs stays in the view, hidden by the sheet in the
  browser. What the browser alone shows, or says otherwise, the view chooses by `w.shell` (`crate::words::Shell`).
  `skein-ui-text` joins a paragraph's runs with no separator; a link inside a paragraph is flattened, so a link goes in a slot.
- **A new word** goes in `words.rs` as its neighbours do: the `Word` enum, the list, an unused number, its English. The
  list's bounds in `limits.rs` are derived: raise a derived count by what you add, and say so; that is not a budget. A word
  nothing uses any more goes.
- **A new token** goes in `look.rs` as its neighbours do, its name in `look::NAMES` and `tests/look.rs` where it lists
  names. A node carries two tokens at most.

## Look requests

`configure-chat-draft` and `new-menu` are landed by this plan, by the procedure in
`temper/docs/design/draft/look-requests.md`, sections 3 to 5; its section 3's steps are spread across the tasks that
name the request. **Each commit of a task that lands a request's markup or rules carries its trailer directly above
`Co-Authored-By:`,** as `Look-request: configure-chat-draft` or `Look-request: new-menu`. A task's spec says whether it does.
No other task's commit carries one. The drawing (the flow, its figure and its `skein-ui-flow-*` tokens) is left out of
`configure-chat-draft`: `skein/docs/design/draft/diagrams.md` waits for the user.

## Generated files are yours

In the same task, regenerate whatever your change makes stale, rather than hand-merging it:

- The terminal's goldens: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal --lib`, then read
  the diff: each change must keep its print's meaning.
- The journeys' prints: `heavy cargo run -p temper-client-journeys`, committing the pages it changed. A recording goes
  stale only if the exchange with the engine changes (`temper/docs/design/client/journeys.md`, 8); leave a stale one to
  `chat-draft-journey` and say so.
- The gallery's `names.txt`, kept in its order, and `temper/tools/gallery/src/tests.rs`.
- Schemas' generated code, goldens and `docs/help/reference/*`: by their own tool, as your spec names it.

## Your checks

Besides the implementer's text:

- temper's client packages are `temper-client-domain`, `temper-client-view`, `temper-client-shell-terminal`,
  `temper-client-shell-web`, `temper-client-gallery`, `temper-client-journeys` and `temper-client-world`. A task that
  changes what a page draws runs all of them but those its spec leaves out.
- A task that edits design text runs `python3 -I -B tools/citations/citations.py --base origin/main`.
- `temper-client-browser-world` is a slow tier (`browser-temper`). Your spec names what you run of it. Its tests read the
  page by role and name: `rg` them for what you change, and change those you make stale, whether or not you run them,
  saying which you did not run.

## Your report

Besides what the implementer's text asks: one line for each rule of the request your task landed, and each difference from
the mock or the look request that remains, with why.
