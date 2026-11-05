# ui-chats: the rules every task names

The request is `temper/docs/plan/ui-chats/inputs/request.md`: read its opening, *The mock*, its section
your spec names, and *For every task of this plan*, which binds you and is not repeated here. Where
this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit only the status entries it closes
  (`README.status.md`, `browser.status.md`, `journeys.status.md`, skein's `ui.status.md` and
  `ui-web.status.md`) and the journeys' prints. If the code teaches that a design line is wrong, make
  the small edit, and name it in your report.
- **Decide, don't ask:** a `Question:` is only for what truly blocks the task. A stored format, a
  protocol or a public interface may change freely. A small edit just outside your files (a re-export,
  a `mod` line, a design line, a golden) is made and named in your report.
- **Render proofs:** a throwaway render script is allowed, untracked and gone before the push.
  Screenshots go to `~/src/fabrig/ui-chats/<your task>/` on your builder, outside any worktree, never
  committed; your report names the builder.

## The look

- **The rule to copy** is the selector the request names, in
  `temper/docs/design/client/greenfield/mock.css`, pixel for pixel, onto the client's own classes, in
  the `--gf-*` tokens. The mock's markup is `temper/docs/design/client/greenfield/app.js`: `chatsPage`,
  `chatPage`, `composer`, `render`. The phone's rules are that file's last `@media (max-width: 760px)`.
- **`temper.css`:** new rules go under `/* ==== ui: chats ==== */`, never at the end of the file. Remove
  replaced rules where they stand.
- **Rendering the mock:** the commands in the request's *The mock*; 420 px through CDP's
  `Emulation.setDeviceMetricsOverride`; handle `Page.javascriptDialogOpening`.
- **The terminal is the same view.** What the mock lacks and a print needs stays in the view, hidden by
  the sheet in the browser. What the browser alone shows, or says otherwise, the view chooses by
  `w.shell` (`crate::words::Shell`), as `pages/home.rs`' `step` does. `skein-ui-text` joins a paragraph's
  runs with no separator; a link inside a paragraph is flattened, so a link goes in a slot.
- **A new word** goes in `words.rs` as its neighbours do: the `Word` enum, the list, an unused number, its
  English. The list's bounds in `limits.rs` are derived: raise a derived count by what you add, and say
  so; that is not a budget. A word nothing uses any more goes.
- **A new token** goes in `look.rs` as its neighbours do, its name in `look::NAMES` and
  `tests/look.rs` where it lists names. A node carries two tokens at most.

## Generated files are yours

In the same task, regenerate whatever your change makes stale, rather than hand-merging it:

- The terminal's goldens: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal
  --lib`, then read the diff: each change must keep its print's meaning.
- The journeys' prints: `heavy cargo run -p temper-client-journeys`, committing the pages it changed. A
  recording goes stale only if the exchange with the engine changes (`journeys.md`, 8); leave a stale one
  to `playground-recording` and say so.
- The gallery's `names.txt`, kept in its order, and `tools/gallery/src/tests.rs`.

## Your checks

Besides the implementer's text:

- The tree's packages are `temper-client-domain`, `temper-client-view`, `temper-client-shell-terminal`,
  `temper-client-shell-web`, `temper-client-gallery`, `temper-client-journeys` and `temper-client-world`.
  A task that changes what a page draws runs all of them but those its spec leaves out.
- A task that edits design text runs `python3 -I -B tools/citations/citations.py --base origin/main`.
- `temper-client-browser-world` is a slow tier (`browser-temper`). Your spec names what you run of it.
  Its tests read the page by role and name: `rg` them for what you change, and change those you make
  stale, whether or not you run them, saying which you did not run.

## Your report

Besides what the implementer's text asks: one line for each rule of the request your task landed, and
each difference from the mock that remains, with why.

No commit carries a `Look-request:` trailer: no look request lands here.
